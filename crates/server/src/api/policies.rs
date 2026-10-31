use itertools::Itertools;

use unitycatalog_common::models::policies::v1::policy_info::Options;
use unitycatalog_common::models::policies::v1::*;
use unitycatalog_common::models::{
    ObjectLabel, Resource, ResourceIdent, ResourceName, ResourceRef,
};

use super::{RequestContext, SecuredAction, UpdateMask};
use crate::Result;
use crate::policy::{Permission, Policy, process_resources};
use crate::services::ServerHandler;
use crate::store::{ResourceStore, ResourceStoreReader};
pub use unitycatalog_handlers::policies::PolicyHandler;

/// The securable types a policy can be defined on, in their canonical (upper-case) form.
const SECURABLE_TYPES: [&str; 3] = ["CATALOG", "SCHEMA", "TABLE"];

/// Normalize a securable type from a path or body to its canonical upper-case form.
///
/// Matching is case-insensitive so `table` and `TABLE` address the same securable.
fn securable_type(raw: &str) -> Result<&'static str> {
    SECURABLE_TYPES
        .into_iter()
        .find(|t| t.eq_ignore_ascii_case(raw))
        .ok_or_else(|| {
            crate::Error::invalid_argument(format!(
                "unsupported securable type '{raw}': expected CATALOG, SCHEMA or TABLE"
            ))
        })
}

/// A securable a policy can be defined on, together with its ancestors.
///
/// Ordered from the securable itself outward: a table's chain is
/// `[(TABLE, catalog.schema.table), (SCHEMA, catalog.schema), (CATALOG, catalog)]`.
fn securable_chain(
    on_securable_type: &str,
    on_securable_fullname: &str,
) -> Vec<(&'static str, String)> {
    let segments: Vec<&str> = on_securable_fullname.split('.').collect();
    let levels: &[&str] = match on_securable_type {
        "TABLE" => &["TABLE", "SCHEMA", "CATALOG"],
        "SCHEMA" => &["SCHEMA", "CATALOG"],
        "CATALOG" => &["CATALOG"],
        _ => &[],
    };
    levels
        .iter()
        .enumerate()
        .filter_map(|(i, level)| {
            let take = segments.len().checked_sub(i)?;
            (take > 0).then(|| (*level, segments[..take].join(".")))
        })
        .collect()
}

fn matches_securable(
    policy: &PolicyInfo,
    on_securable_type: &str,
    on_securable_fullname: &str,
) -> bool {
    policy
        .on_securable_type
        .eq_ignore_ascii_case(on_securable_type)
        && policy.on_securable_fullname == on_securable_fullname
}

fn validate_policy_shape(policy: &PolicyInfo) -> Result<()> {
    let policy_type = policy
        .policy_type
        .as_known()
        .ok_or_else(|| crate::Error::invalid_argument("unrecognized policy_type"))?;
    let invalid = |msg: &str| Err(crate::Error::invalid_argument(msg));
    match (policy_type, &policy.options) {
        (PolicyType::POLICY_TYPE_UNSPECIFIED, _) => invalid(
            "policy_type must be one of POLICY_TYPE_ROW_FILTER, POLICY_TYPE_COLUMN_MASK, \
             POLICY_TYPE_DENY or POLICY_TYPE_GRANT",
        ),
        (PolicyType::POLICY_TYPE_ROW_FILTER, Some(Options::RowFilter(r)))
            if !r.function_name.is_empty() =>
        {
            Ok(())
        }
        (PolicyType::POLICY_TYPE_ROW_FILTER, _) => {
            invalid("row filter policies require row_filter.function_name")
        }
        (PolicyType::POLICY_TYPE_COLUMN_MASK, _) if policy.match_columns.is_empty() => {
            invalid("column mask policies require at least one match_columns entry")
        }
        (PolicyType::POLICY_TYPE_COLUMN_MASK, Some(Options::ColumnMask(m)))
            if !m.function_name.is_empty() =>
        {
            Ok(())
        }
        (PolicyType::POLICY_TYPE_COLUMN_MASK, _) => {
            invalid("column mask policies require column_mask.function_name")
        }
        (PolicyType::POLICY_TYPE_DENY, Some(Options::Deny(_)))
        | (PolicyType::POLICY_TYPE_GRANT, Some(Options::Grant(_))) => Ok(()),
        (PolicyType::POLICY_TYPE_DENY, _) => invalid("deny policies require deny options"),
        (PolicyType::POLICY_TYPE_GRANT, _) => invalid("grant policies require grant options"),
    }
}

/// Collapse the per-kind option fields of a create/update request into the stored oneof.
///
/// Requests carry `row_filter`, `column_mask`, `deny` and `grant` as separate fields (the
/// same JSON as the oneof on [`PolicyInfo`]); at most one may be set.
fn request_options(
    row_filter: Option<FunctionRef>,
    column_mask: Option<FunctionRef>,
    deny: Option<PrivilegeOptions>,
    grant: Option<PrivilegeOptions>,
) -> Result<Option<Options>> {
    let options: Vec<Options> = [
        row_filter.map(|f| Options::RowFilter(Box::new(f))),
        column_mask.map(|f| Options::ColumnMask(Box::new(f))),
        deny.map(|p| Options::Deny(Box::new(p))),
        grant.map(|p| Options::Grant(Box::new(p))),
    ]
    .into_iter()
    .flatten()
    .collect();
    match <[Options; 1]>::try_from(options) {
        Ok([options]) => Ok(Some(options)),
        Err(options) if options.is_empty() => Ok(None),
        Err(_) => Err(crate::Error::invalid_argument(
            "at most one of row_filter, column_mask, deny and grant may be set",
        )),
    }
}

/// Apply an update request onto the stored policy.
///
/// With an empty or `*` update mask every field set in the request is applied. With an
/// explicit mask only the listed fields are applied, and a listed field that is unset in
/// the request is cleared. The enforcement options (`row_filter`, `column_mask`, `deny`,
/// `grant`) are a single oneof, so naming any of them replaces the options as a whole.
fn apply_update(policy: &mut PolicyInfo, request: UpdatePolicyRequest) -> Result<()> {
    let options = request_options(
        request.row_filter.into_option(),
        request.column_mask.into_option(),
        request.deny.into_option(),
        request.grant.into_option(),
    )?;
    let mask = UpdateMask::parse(
        request.update_mask.as_deref(),
        &[
            "comment",
            "to_principals",
            "except_principals",
            "for_securable_type",
            "when_condition",
            "match_columns",
            "policy_type",
            "row_filter",
            "column_mask",
            "deny",
            "grant",
        ],
    )?;
    let masked = |field: &str| mask.names(field);

    if matches!(mask, UpdateMask::All) {
        if request.comment.is_some() {
            policy.comment = request.comment;
        }
        if !request.to_principals.is_empty() {
            policy.to_principals = request.to_principals;
        }
        if !request.except_principals.is_empty() {
            policy.except_principals = request.except_principals;
        }
        if request.for_securable_type.is_some() {
            policy.for_securable_type = request.for_securable_type;
        }
        if request.when_condition.is_some() {
            policy.when_condition = request.when_condition;
        }
        if !request.match_columns.is_empty() {
            policy.match_columns = request.match_columns;
        }
        if let Some(policy_type) = request.policy_type {
            policy.policy_type = policy_type;
        }
        if options.is_some() {
            policy.options = options;
        }
        return Ok(());
    }

    if masked("comment") {
        policy.comment = request.comment;
    }
    if masked("to_principals") {
        policy.to_principals = request.to_principals;
    }
    if masked("except_principals") {
        policy.except_principals = request.except_principals;
    }
    if masked("for_securable_type") {
        policy.for_securable_type = request.for_securable_type;
    }
    if masked("when_condition") {
        policy.when_condition = request.when_condition;
    }
    if masked("match_columns") {
        policy.match_columns = request.match_columns;
    }
    if masked("policy_type") {
        policy.policy_type = request.policy_type.unwrap_or_default();
    }
    if ["row_filter", "column_mask", "deny", "grant"]
        .iter()
        .any(|f| masked(f))
    {
        policy.options = options;
    }
    Ok(())
}

#[async_trait::async_trait]
impl PolicyHandler<RequestContext> for ServerHandler<RequestContext> {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_policy(
        &self,
        request: CreatePolicyRequest,
        context: RequestContext,
    ) -> Result<PolicyInfo> {
        self.check_required(&request, &context).await?;
        let for_securable_type = request
            .for_securable_type
            .as_deref()
            .map(securable_type)
            .transpose()?
            .map(str::to_string);
        let policy = PolicyInfo {
            name: request.name,
            on_securable_type: securable_type(&request.on_securable_type)?.to_string(),
            on_securable_fullname: request.on_securable_fullname,
            policy_type: request.policy_type,
            comment: request.comment,
            to_principals: request.to_principals,
            except_principals: request.except_principals,
            for_securable_type,
            when_condition: request.when_condition,
            match_columns: request.match_columns,
            options: request_options(
                request.row_filter.into_option(),
                request.column_mask.into_option(),
                request.deny.into_option(),
                request.grant.into_option(),
            )?,
            ..Default::default()
        };
        validate_policy_shape(&policy)?;
        tracing::Span::current().record("resource_name", &policy.name);

        // Name uniqueness is scoped to the securable, not global.
        let existing = self
            .list(&ObjectLabel::PolicyInfo, None, None, None)
            .await?
            .0;
        let clashes = existing.into_iter().any(|r| {
            let Ok(p): std::result::Result<PolicyInfo, _> = r.try_into() else {
                return false;
            };
            matches_securable(&p, &policy.on_securable_type, &policy.on_securable_fullname)
                && p.name == policy.name
        });
        if clashes {
            return Err(crate::Error::invalid_argument(format!(
                "policy '{}' already exists on {} '{}'",
                policy.name, policy.on_securable_type, policy.on_securable_fullname
            )));
        }

        Ok(self.create(policy.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_policy(
        &self,
        request: DeletePolicyRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = find_policy_ident(
            self,
            securable_type(&request.on_securable_type)?,
            &request.on_securable_fullname,
            &request.name,
        )
        .await?;
        Ok(self.delete(&ident).await?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_policy(
        &self,
        request: GetPolicyRequest,
        context: RequestContext,
    ) -> Result<PolicyInfo> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = find_policy_ident(
            self,
            securable_type(&request.on_securable_type)?,
            &request.on_securable_fullname,
            &request.name,
        )
        .await?;
        Ok(self.get(&ident).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_policies(
        &self,
        request: ListPoliciesRequest,
        context: RequestContext,
    ) -> Result<ListPoliciesResponse> {
        self.check_required(&request, &context).await?;
        let on_securable_type = securable_type(&request.on_securable_type)?;

        let (all, _) = self
            .list(&ObjectLabel::PolicyInfo, None, None, None)
            .await?;
        let mut policies: Vec<PolicyInfo> = all.into_iter().map(|r| r.try_into()).try_collect()?;

        if request.include_inherited.unwrap_or(false) {
            let chain = securable_chain(on_securable_type, &request.on_securable_fullname);
            policies.retain(|p| chain.iter().any(|(t, name)| matches_securable(p, t, name)));
        } else {
            policies.retain(|p| {
                matches_securable(p, on_securable_type, &request.on_securable_fullname)
            });
        }

        let mut resources: Vec<Resource> = policies.into_iter().map(Resource::PolicyInfo).collect();
        process_resources(self, &context, &Permission::Read, &mut resources).await?;

        let max_results = super::page_size(request.max_results)?.unwrap_or(resources.len());
        let offset = request
            .page_token
            .as_deref()
            .and_then(|t| t.parse::<usize>().ok())
            .unwrap_or(0);
        let next_page_token =
            (offset + max_results < resources.len()).then(|| (offset + max_results).to_string());
        let page = resources
            .into_iter()
            .skip(offset)
            .take(max_results)
            .map(|r| r.try_into())
            .try_collect()?;

        Ok(ListPoliciesResponse {
            policies: page,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_policy(
        &self,
        mut request: UpdatePolicyRequest,
        context: RequestContext,
    ) -> Result<PolicyInfo> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = find_policy_ident(
            self,
            securable_type(&request.on_securable_type)?,
            &request.on_securable_fullname,
            &request.name,
        )
        .await?;
        if let Some(t) = request.for_securable_type.as_deref() {
            request.for_securable_type = Some(securable_type(t)?.to_string());
        }
        let mut policy: PolicyInfo = self.get(&ident).await?.0.try_into()?;
        apply_update(&mut policy, request)?;
        validate_policy_shape(&policy)?;
        Ok(self.update(&ident, policy.into()).await?.0.try_into()?)
    }
}

/// Resolve the [`ResourceIdent`] of the policy matching `name` on the given securable.
///
/// Policies are stored as flat objects keyed by their own `name`, so this scans all
/// `PolicyInfo` objects rather than resolving via a hierarchical `ResourceName`.
async fn find_policy_ident<S: ResourceStore>(
    store: &S,
    on_securable_type: &str,
    on_securable_fullname: &str,
    name: &str,
) -> Result<ResourceIdent> {
    let (all, _) = store
        .list(&ObjectLabel::PolicyInfo, None, None, None)
        .await?;
    all.into_iter()
        .find_map(|r| {
            let p: PolicyInfo = r.try_into().ok()?;
            (matches_securable(&p, on_securable_type, on_securable_fullname) && p.name == name)
                .then(|| ResourceIdent::policy_info(ResourceName::new([name])))
        })
        .ok_or(crate::Error::NotFound)
}

impl SecuredAction for CreatePolicyRequest {
    fn resource(&self) -> ResourceIdent {
        securable_ident(&self.on_securable_type, &self.on_securable_fullname)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for ListPoliciesRequest {
    fn resource(&self) -> ResourceIdent {
        securable_ident(&self.on_securable_type, &self.on_securable_fullname)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetPolicyRequest {
    fn resource(&self) -> ResourceIdent {
        securable_ident(&self.on_securable_type, &self.on_securable_fullname)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdatePolicyRequest {
    fn resource(&self) -> ResourceIdent {
        securable_ident(&self.on_securable_type, &self.on_securable_fullname)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeletePolicyRequest {
    fn resource(&self) -> ResourceIdent {
        securable_ident(&self.on_securable_type, &self.on_securable_fullname)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

/// Authorization for policy management is checked against the securable the policy is
/// defined on, since defining a row-filter/column-mask is a governance action over that
/// securable, not over the policy object itself.
fn securable_ident(on_securable_type: &str, on_securable_fullname: &str) -> ResourceIdent {
    let name = ResourceName::from_naive_str_split(on_securable_fullname);
    match securable_type(on_securable_type) {
        Ok("CATALOG") => ResourceIdent::catalog(name),
        Ok("SCHEMA") => ResourceIdent::schema(name),
        Ok("TABLE") => ResourceIdent::table(name),
        _ => ResourceIdent::catalog(ResourceRef::Undefined),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::catalogs::v1::Catalog;
    use unitycatalog_common::models::schemas::v1::Schema;
    use unitycatalog_common::models::tables::v1::{DataSourceFormat, Table, TableType};
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::memory::InMemoryResourceStore;
    use crate::policy::ConstantPolicy;
    use crate::services::ServerHandler;

    async fn handler() -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        store
            .create(
                Catalog {
                    name: "cat".to_string(),
                    ..Default::default()
                }
                .into(),
            )
            .await
            .unwrap();
        store
            .create(
                Schema {
                    name: "sch".to_string(),
                    catalog_name: "cat".to_string(),
                    ..Default::default()
                }
                .into(),
            )
            .await
            .unwrap();
        store
            .create(
                Table {
                    name: "tbl".to_string(),
                    catalog_name: "cat".to_string(),
                    schema_name: "sch".to_string(),
                    full_name: "cat.sch.tbl".to_string(),
                    table_type: TableType::Managed.into(),
                    data_source_format: DataSourceFormat::Delta.into(),
                    ..Default::default()
                }
                .into(),
            )
            .await
            .unwrap();
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        ServerHandler::try_new_tokio(policy, store).unwrap()
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: crate::policy::Principal::anonymous(),
        }
    }

    fn row_filter_request(
        name: &str,
        on_securable_type: &str,
        on_securable_fullname: &str,
    ) -> CreatePolicyRequest {
        CreatePolicyRequest {
            name: name.to_string(),
            on_securable_type: on_securable_type.to_string(),
            on_securable_fullname: on_securable_fullname.to_string(),
            policy_type: PolicyType::RowFilter.into(),
            to_principals: vec!["group:analysts".to_string()],
            row_filter: Some(FunctionRef {
                function_name: "cat.sch.my_filter".to_string(),
                ..Default::default()
            })
            .into(),
            ..Default::default()
        }
    }

    fn update_request(name: &str) -> UpdatePolicyRequest {
        UpdatePolicyRequest {
            on_securable_type: "TABLE".to_string(),
            on_securable_fullname: "cat.sch.tbl".to_string(),
            name: name.to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn policy_crud_round_trip() {
        let h = handler().await;

        let created = h
            .create_policy(row_filter_request("p1", "TABLE", "cat.sch.tbl"), ctx())
            .await
            .unwrap();
        assert_eq!(created.name, "p1");
        assert_eq!(created.on_securable_fullname, "cat.sch.tbl");

        let fetched = h
            .get_policy(
                GetPolicyRequest {
                    on_securable_type: "table".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    name: "p1".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(fetched.name, "p1");

        let listed = h
            .list_policies(
                ListPoliciesRequest {
                    on_securable_type: "table".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    include_inherited: None,
                    max_results: None,
                    page_token: None,
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(listed.policies.len(), 1);

        let updated = h
            .update_policy(
                UpdatePolicyRequest {
                    comment: Some("updated".to_string()),
                    ..update_request("p1")
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(updated.comment.as_deref(), Some("updated"));

        h.delete_policy(
            DeletePolicyRequest {
                on_securable_type: "table".to_string(),
                on_securable_fullname: "cat.sch.tbl".to_string(),
                name: "p1".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();

        let missing = h
            .get_policy(
                GetPolicyRequest {
                    on_securable_type: "table".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    name: "p1".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(missing.is_err(), "expected NotFound after delete");
    }

    #[tokio::test]
    async fn list_include_inherited_unions_ancestor_chain() {
        let h = handler().await;

        h.create_policy(
            row_filter_request("catalog_policy", "CATALOG", "cat"),
            ctx(),
        )
        .await
        .unwrap();
        h.create_policy(
            row_filter_request("schema_policy", "SCHEMA", "cat.sch"),
            ctx(),
        )
        .await
        .unwrap();
        h.create_policy(
            row_filter_request("table_policy", "TABLE", "cat.sch.tbl"),
            ctx(),
        )
        .await
        .unwrap();

        let without_inheritance = h
            .list_policies(
                ListPoliciesRequest {
                    on_securable_type: "table".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    include_inherited: None,
                    max_results: None,
                    page_token: None,
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(without_inheritance.policies.len(), 1);
        assert_eq!(without_inheritance.policies[0].name, "table_policy");

        let with_inheritance = h
            .list_policies(
                ListPoliciesRequest {
                    on_securable_type: "table".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    include_inherited: Some(true),
                    max_results: None,
                    page_token: None,
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        let mut names: Vec<_> = with_inheritance
            .policies
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        names.sort();
        assert_eq!(
            names,
            vec!["catalog_policy", "schema_policy", "table_policy"]
        );
        // Each entry still carries its own securable, so the caller can see where it came from.
        let by_name = |n: &str| {
            with_inheritance
                .policies
                .iter()
                .find(|p| p.name == n)
                .unwrap()
        };
        assert_eq!(by_name("catalog_policy").on_securable_fullname, "cat");
        assert_eq!(by_name("schema_policy").on_securable_fullname, "cat.sch");
        assert_eq!(by_name("table_policy").on_securable_fullname, "cat.sch.tbl");
    }

    #[tokio::test]
    async fn create_rejects_duplicate_name_on_same_securable() {
        let h = handler().await;
        h.create_policy(row_filter_request("p1", "TABLE", "cat.sch.tbl"), ctx())
            .await
            .unwrap();

        let result = h
            .create_policy(row_filter_request("p1", "TABLE", "cat.sch.tbl"), ctx())
            .await;
        assert!(
            result.is_err(),
            "duplicate policy name on same securable must be rejected"
        );
    }

    #[tokio::test]
    async fn create_rejects_column_mask_without_match_columns() {
        let h = handler().await;
        let request = CreatePolicyRequest {
            policy_type: PolicyType::ColumnMask.into(),
            row_filter: None.into(),
            column_mask: Some(FunctionRef {
                function_name: "cat.sch.my_mask".to_string(),
                on_column: Some("ssn".to_string()),
                ..Default::default()
            })
            .into(),
            ..row_filter_request("bad_mask", "TABLE", "cat.sch.tbl")
        };
        let result = h.create_policy(request, ctx()).await;
        assert!(
            result.is_err(),
            "column mask without match_columns must be rejected"
        );
    }

    #[tokio::test]
    async fn get_unknown_policy_is_not_found() {
        let h = handler().await;
        let result = h
            .get_policy(
                GetPolicyRequest {
                    on_securable_type: "table".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    name: "does-not-exist".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(result.is_err(), "unknown policy must be NotFound");
    }

    #[tokio::test]
    async fn update_mask_clears_listed_fields_and_keeps_others() {
        let h = handler().await;
        let mut request = row_filter_request("p1", "TABLE", "cat.sch.tbl");
        request.comment = Some("original".to_string());
        request.when_condition = Some("hasTag('pii')".to_string());
        h.create_policy(request, ctx()).await.unwrap();

        // Without a mask only the fields set in the request change.
        let updated = h
            .update_policy(
                UpdatePolicyRequest {
                    comment: Some("changed".to_string()),
                    ..update_request("p1")
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(updated.comment.as_deref(), Some("changed"));
        assert_eq!(updated.when_condition.as_deref(), Some("hasTag('pii')"));
        assert_eq!(updated.to_principals, vec!["group:analysts".to_string()]);

        // A field named in the mask but unset in the request is cleared.
        let cleared = h
            .update_policy(
                UpdatePolicyRequest {
                    update_mask: Some("when_condition".to_string()),
                    ..update_request("p1")
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(cleared.when_condition, None);
        assert_eq!(cleared.comment.as_deref(), Some("changed"));

        let unknown = h
            .update_policy(
                UpdatePolicyRequest {
                    update_mask: Some("name".to_string()),
                    ..update_request("p1")
                },
                ctx(),
            )
            .await;
        assert!(
            unknown.is_err(),
            "masking a non-updatable field must be rejected"
        );
    }

    #[tokio::test]
    async fn deny_policy_requires_deny_options() {
        let h = handler().await;
        let missing = CreatePolicyRequest {
            policy_type: PolicyType::Deny.into(),
            row_filter: None.into(),
            ..row_filter_request("deny_missing", "CATALOG", "cat")
        };
        assert!(h.create_policy(missing, ctx()).await.is_err());

        let created = h
            .create_policy(
                CreatePolicyRequest {
                    policy_type: PolicyType::Deny.into(),
                    row_filter: None.into(),
                    deny: Some(PrivilegeOptions {
                        privileges: vec!["SELECT".to_string()],
                        ..Default::default()
                    })
                    .into(),
                    ..row_filter_request("deny_select", "catalog", "cat")
                },
                ctx(),
            )
            .await
            .unwrap();
        // The securable type is stored in its canonical upper-case form.
        assert_eq!(created.on_securable_type, "CATALOG");
    }
}
