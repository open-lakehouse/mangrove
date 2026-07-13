use itertools::Itertools;

use unitycatalog_common::models::policies::v1::policy_info::Function;
use unitycatalog_common::models::policies::v1::*;
use unitycatalog_common::models::{
    ObjectLabel, Resource, ResourceIdent, ResourceName, ResourceRef,
};

use super::{RequestContext, SecuredAction};
use crate::Result;
pub use crate::codegen::policies::PolicyHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::store::ResourceStore;

/// A securable a policy can be defined on, together with its ancestors.
///
/// Ordered from the securable itself outward: a table's chain is
/// `[(tables, catalog.schema.table), (schemas, catalog.schema), (catalogs, catalog)]`.
fn securable_chain(
    on_securable_type: &str,
    on_securable_fullname: &str,
) -> Vec<(&'static str, String)> {
    let segments: Vec<&str> = on_securable_fullname.split('.').collect();
    let levels: &[&str] = match on_securable_type {
        "tables" => &["tables", "schemas", "catalogs"],
        "schemas" => &["schemas", "catalogs"],
        "catalogs" => &["catalogs"],
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
    policy.on_securable_type == on_securable_type
        && policy.on_securable_fullname == on_securable_fullname
}

fn validate_policy_shape(policy: &PolicyInfo) -> Result<()> {
    let policy_type = policy
        .policy_type
        .as_known()
        .ok_or_else(|| crate::Error::invalid_argument("unrecognized policy_type"))?;
    match policy_type {
        PolicyType::POLICY_TYPE_COLUMN_MASK if policy.match_columns.is_empty() => {
            Err(crate::Error::invalid_argument(
                "column mask policies require at least one match_columns entry",
            ))
        }
        PolicyType::POLICY_TYPE_ROW_FILTER
            if policy
                .function
                .as_ref()
                .and_then(|f| match f {
                    Function::RowFilter(r) => Some(r),
                    _ => None,
                })
                .is_none_or(|r| r.function_name.is_empty()) =>
        {
            Err(crate::Error::invalid_argument(
                "row filter policies require row_filter.function_name",
            ))
        }
        PolicyType::POLICY_TYPE_UNSPECIFIED => Err(crate::Error::invalid_argument(
            "policy_type must be POLICY_TYPE_ROW_FILTER or POLICY_TYPE_COLUMN_MASK",
        )),
        _ => Ok(()),
    }
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> PolicyHandler<RequestContext> for T {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_policy(
        &self,
        request: CreatePolicyRequest,
        context: RequestContext,
    ) -> Result<PolicyInfo> {
        self.check_required(&request, &context).await?;
        let mut policy = request
            .policy_info
            .ok_or_else(|| crate::Error::invalid_argument("policy_info must be provided"))?;
        policy.on_securable_type = request.on_securable_type;
        policy.on_securable_fullname = request.on_securable_fullname;
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
            &request.on_securable_type,
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
            &request.on_securable_type,
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

        let (all, _) = self
            .list(&ObjectLabel::PolicyInfo, None, None, None)
            .await?;
        let mut policies: Vec<PolicyInfo> = all.into_iter().map(|r| r.try_into()).try_collect()?;

        if request.include_inherited.unwrap_or(false) {
            let chain = securable_chain(&request.on_securable_type, &request.on_securable_fullname);
            policies.retain(|p| chain.iter().any(|(t, name)| matches_securable(p, t, name)));
        } else {
            policies.retain(|p| {
                matches_securable(
                    p,
                    &request.on_securable_type,
                    &request.on_securable_fullname,
                )
            });
        }

        let mut resources: Vec<Resource> = policies.into_iter().map(Resource::PolicyInfo).collect();
        process_resources(self, &context, &Permission::Read, &mut resources).await?;

        let max_results = request
            .max_results
            .map(|v| v as usize)
            .unwrap_or(resources.len());
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
        request: UpdatePolicyRequest,
        context: RequestContext,
    ) -> Result<PolicyInfo> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = find_policy_ident(
            self,
            &request.on_securable_type,
            &request.on_securable_fullname,
            &request.name,
        )
        .await?;
        let mut policy = request
            .policy_info
            .ok_or_else(|| crate::Error::invalid_argument("policy_info must be provided"))?;
        // The name and securable are the resource identity and are taken from the path, not the body.
        policy.name = request.name;
        policy.on_securable_type = request.on_securable_type;
        policy.on_securable_fullname = request.on_securable_fullname;
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
    match on_securable_type {
        "catalogs" => ResourceIdent::catalog(name),
        "schemas" => ResourceIdent::schema(name),
        "tables" => ResourceIdent::table(name),
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

    fn row_filter_policy(
        name: &str,
        on_securable_type: &str,
        on_securable_fullname: &str,
    ) -> PolicyInfo {
        PolicyInfo {
            name: name.to_string(),
            on_securable_type: on_securable_type.to_string(),
            on_securable_fullname: on_securable_fullname.to_string(),
            policy_type: PolicyType::RowFilter.into(),
            to_principals: vec!["group:analysts".to_string()],
            function: Some(Function::RowFilter(Box::new(FunctionRef {
                function_name: "cat.sch.my_filter".to_string(),
                using: vec![],
                ..Default::default()
            }))),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn policy_crud_round_trip() {
        let h = handler().await;

        let created = h
            .create_policy(
                CreatePolicyRequest {
                    on_securable_type: "tables".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    policy_info: Some(row_filter_policy("p1", "", "")).into(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(created.name, "p1");
        assert_eq!(created.on_securable_fullname, "cat.sch.tbl");

        let fetched = h
            .get_policy(
                GetPolicyRequest {
                    on_securable_type: "tables".to_string(),
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
                    on_securable_type: "tables".to_string(),
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

        let mut updated_policy = row_filter_policy("p1", "", "");
        updated_policy.comment = Some("updated".to_string());
        let updated = h
            .update_policy(
                UpdatePolicyRequest {
                    on_securable_type: "tables".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    name: "p1".to_string(),
                    policy_info: Some(updated_policy).into(),
                    update_mask: None,
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(updated.comment.as_deref(), Some("updated"));

        h.delete_policy(
            DeletePolicyRequest {
                on_securable_type: "tables".to_string(),
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
                    on_securable_type: "tables".to_string(),
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
            CreatePolicyRequest {
                on_securable_type: "catalogs".to_string(),
                on_securable_fullname: "cat".to_string(),
                policy_info: Some(row_filter_policy("catalog_policy", "", "")).into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_policy(
            CreatePolicyRequest {
                on_securable_type: "schemas".to_string(),
                on_securable_fullname: "cat.sch".to_string(),
                policy_info: Some(row_filter_policy("schema_policy", "", "")).into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_policy(
            CreatePolicyRequest {
                on_securable_type: "tables".to_string(),
                on_securable_fullname: "cat.sch.tbl".to_string(),
                policy_info: Some(row_filter_policy("table_policy", "", "")).into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();

        let without_inheritance = h
            .list_policies(
                ListPoliciesRequest {
                    on_securable_type: "tables".to_string(),
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
                    on_securable_type: "tables".to_string(),
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
        h.create_policy(
            CreatePolicyRequest {
                on_securable_type: "tables".to_string(),
                on_securable_fullname: "cat.sch.tbl".to_string(),
                policy_info: Some(row_filter_policy("p1", "", "")).into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();

        let result = h
            .create_policy(
                CreatePolicyRequest {
                    on_securable_type: "tables".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    policy_info: Some(row_filter_policy("p1", "", "")).into(),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(
            result.is_err(),
            "duplicate policy name on same securable must be rejected"
        );
    }

    #[tokio::test]
    async fn create_rejects_column_mask_without_match_columns() {
        let h = handler().await;
        let mut policy = row_filter_policy("bad_mask", "", "");
        policy.policy_type = PolicyType::ColumnMask.into();
        policy.function = Some(Function::ColumnMask(Box::new(FunctionRef {
            function_name: "cat.sch.my_mask".to_string(),
            using: vec![],
            ..Default::default()
        })));
        let result = h
            .create_policy(
                CreatePolicyRequest {
                    on_securable_type: "tables".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    policy_info: Some(policy).into(),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
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
                    on_securable_type: "tables".to_string(),
                    on_securable_fullname: "cat.sch.tbl".to_string(),
                    name: "does-not-exist".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(result.is_err(), "unknown policy must be NotFound");
    }
}
