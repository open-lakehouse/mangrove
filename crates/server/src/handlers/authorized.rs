//! Policy-checking decorator.
//!
//! [`Authorized`] wraps any per-resource handler and enforces this server's
//! [`Policy`] in front of it: every request is policy-checked before it is
//! delegated, and `list_*` responses are filtered on [`Permission::Read`], so the
//! wrapped handler never returns resources the caller is not allowed to see.
//!
//! The inner handler is typically a leaf that does no authorization of its own.
//! With the `proxy` feature, the `olai-uc-client` service clients implement the
//! handler traits by forwarding to an upstream Unity Catalog instance, so
//! `Authorized::new(client.catalogs_client(), policy)` serves catalogs from the
//! upstream under the local policy, which is how the hybrid server proxies a
//! surface.
//!
//! Implemented for every handler trait whose requests implement
//! [`SecuredAction`](crate::api::SecuredAction). Temporary credentials are not
//! covered: their authorization resolves the securable first.

use std::sync::Arc;

use async_trait::async_trait;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
use unitycatalog_common::models::agents::v0alpha1::*;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::external_locations::v1::*;
use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::model_versions::v1::*;
use unitycatalog_common::models::policies::v1::*;
use unitycatalog_common::models::registered_models::v1::*;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::staging_tables::v1::*;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::volumes::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName};

use crate::Result;
use crate::api::{
    AgentHandler, AgentSkillHandler, CatalogHandler, CredentialHandler, EntityTagAssignmentHandler,
    ExternalLocationHandler, FunctionHandler, ModelVersionHandler, PolicyHandler,
    RegisteredModelHandler, SchemaHandler, StagingTableHandler, TableHandler, TagPolicyHandler,
    VolumeHandler,
};
use crate::policy::{Decision, Permission, Policy, filter_authorized};

/// A handler that checks every request against a [`Policy`] before delegating
/// to `inner`, and filters `list_*` responses down to readable resources.
pub struct Authorized<H, Cx> {
    inner: H,
    policy: Arc<dyn Policy<Cx>>,
}

impl<H, Cx> Authorized<H, Cx> {
    pub fn new(inner: H, policy: Arc<dyn Policy<Cx>>) -> Self {
        Self { inner, policy }
    }

    /// The wrapped handler.
    pub fn inner(&self) -> &H {
        &self.inner
    }
}

impl<H: Clone, Cx> Clone for Authorized<H, Cx> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            policy: self.policy.clone(),
        }
    }
}

/// Keep the `items` whose ident (per `ident`) the policy allows reading, for
/// list items that do not implement `ResourceExt`.
async fn retain_authorized<Cx: Send + Sync + 'static, R>(
    policy: &dyn Policy<Cx>,
    context: &Cx,
    items: &mut Vec<R>,
    ident: fn(&R) -> ResourceIdent,
) -> Result<()> {
    let idents: Vec<_> = items.iter().map(ident).collect();
    let decisions = policy
        .authorize_many(&idents, &Permission::Read, context)
        .await?;
    // `decisions[i]` corresponds to `items[i]`; pair them in forward order.
    let mut allow = decisions.into_iter().map(|d| d == Decision::Allow);
    items.retain(|_| allow.next().unwrap_or(false));
    Ok(())
}

/// `TableSummary` carries only the full name, so rebuild the table ident from
/// it (mirrors the `GetTableRequest` secured action).
fn table_summary_ident(summary: &TableSummary) -> ResourceIdent {
    ResourceIdent::table(ResourceName::from_naive_str_split(&summary.full_name))
}

/// Implement a handler trait for [`Authorized`].
///
/// `list` methods filter the named response field after delegating, through
/// `filter_authorized` or, with `by <fn>`, through [`retain_authorized`]. `fn`
/// methods only check the request.
macro_rules! authorized {
    (
        $trait:ident {
            $(list $list:ident($list_req:ty) -> $list_resp:ty => $field:ident $(by $by:path)?;)*
            $(fn $method:ident($req:ty) -> $resp:ty;)*
        }
    ) => {
        #[async_trait]
        impl<H, Cx> $trait<Cx> for Authorized<H, Cx>
        where
            H: $trait<Cx>,
            Cx: Clone + Send + Sync + 'static,
        {
            $(
                async fn $list(&self, request: $list_req, context: Cx) -> Result<$list_resp> {
                    self.policy.check_required(&request, &context).await?;
                    let mut resp = self.inner.$list(request, context.clone()).await?;
                    authorized!(@filter self, context, &mut resp.$field $(, $by)?);
                    Ok(resp)
                }
            )*
            $(
                async fn $method(&self, request: $req, context: Cx) -> Result<$resp> {
                    self.policy.check_required(&request, &context).await?;
                    self.inner.$method(request, context).await
                }
            )*
        }
    };
    (@filter $self:ident, $context:ident, $items:expr) => {
        filter_authorized(&*$self.policy, &$context, &Permission::Read, $items).await?
    };
    (@filter $self:ident, $context:ident, $items:expr, $by:path) => {
        retain_authorized(&*$self.policy, &$context, $items, $by).await?
    };
}

authorized!(AgentHandler {
    list list_agents(ListAgentsRequest) -> ListAgentsResponse => agents;
    fn create_agent(CreateAgentRequest) -> Agent;
    fn get_agent(GetAgentRequest) -> Agent;
    fn update_agent(UpdateAgentRequest) -> Agent;
    fn delete_agent(DeleteAgentRequest) -> ();
});

authorized!(AgentSkillHandler {
    list list_agent_skills(ListAgentSkillsRequest) -> ListAgentSkillsResponse => agent_skills;
    fn create_agent_skill(CreateAgentSkillRequest) -> AgentSkill;
    fn get_agent_skill(GetAgentSkillRequest) -> AgentSkill;
    fn update_agent_skill(UpdateAgentSkillRequest) -> AgentSkill;
    fn delete_agent_skill(DeleteAgentSkillRequest) -> ();
});

authorized!(CatalogHandler {
    list list_catalogs(ListCatalogsRequest) -> ListCatalogsResponse => catalogs;
    fn create_catalog(CreateCatalogRequest) -> Catalog;
    fn get_catalog(GetCatalogRequest) -> Catalog;
    fn update_catalog(UpdateCatalogRequest) -> Catalog;
    fn delete_catalog(DeleteCatalogRequest) -> ();
});

authorized!(CredentialHandler {
    list list_credentials(ListCredentialsRequest) -> ListCredentialsResponse => credentials;
    fn create_credential(CreateCredentialRequest) -> Credential;
    fn get_credential(GetCredentialRequest) -> Credential;
    fn update_credential(UpdateCredentialRequest) -> Credential;
    fn delete_credential(DeleteCredentialRequest) -> ();
});

// The assignments are all on the one entity the request names, so the request
// check covers them and the list is not filtered.
authorized!(EntityTagAssignmentHandler {
    fn list_entity_tag_assignments(ListEntityTagAssignmentsRequest) -> ListEntityTagAssignmentsResponse;
    fn create_entity_tag_assignment(CreateEntityTagAssignmentRequest) -> EntityTagAssignment;
    fn get_entity_tag_assignment(GetEntityTagAssignmentRequest) -> EntityTagAssignment;
    fn update_entity_tag_assignment(UpdateEntityTagAssignmentRequest) -> EntityTagAssignment;
    fn delete_entity_tag_assignment(DeleteEntityTagAssignmentRequest) -> ();
});

authorized!(ExternalLocationHandler {
    list list_external_locations(ListExternalLocationsRequest) -> ListExternalLocationsResponse => external_locations;
    fn create_external_location(CreateExternalLocationRequest) -> ExternalLocation;
    fn get_external_location(GetExternalLocationRequest) -> ExternalLocation;
    fn update_external_location(UpdateExternalLocationRequest) -> ExternalLocation;
    fn delete_external_location(DeleteExternalLocationRequest) -> ();
});

authorized!(FunctionHandler {
    list list_functions(ListFunctionsRequest) -> ListFunctionsResponse => functions;
    fn create_function(CreateFunctionRequest) -> Function;
    fn get_function(GetFunctionRequest) -> Function;
    fn update_function(UpdateFunctionRequest) -> Function;
    fn delete_function(DeleteFunctionRequest) -> ();
});

authorized!(ModelVersionHandler {
    list list_model_versions(ListModelVersionsRequest) -> ListModelVersionsResponse => model_versions;
    fn create_model_version(CreateModelVersionRequest) -> ModelVersion;
    fn get_model_version(GetModelVersionRequest) -> ModelVersion;
    fn update_model_version(UpdateModelVersionRequest) -> ModelVersion;
    fn delete_model_version(DeleteModelVersionRequest) -> ();
    fn finalize_model_version(FinalizeModelVersionRequest) -> ModelVersion;
});

authorized!(PolicyHandler {
    list list_policies(ListPoliciesRequest) -> ListPoliciesResponse => policies;
    fn create_policy(CreatePolicyRequest) -> PolicyInfo;
    fn get_policy(GetPolicyRequest) -> PolicyInfo;
    fn update_policy(UpdatePolicyRequest) -> PolicyInfo;
    fn delete_policy(DeletePolicyRequest) -> ();
});

authorized!(RegisteredModelHandler {
    list list_registered_models(ListRegisteredModelsRequest) -> ListRegisteredModelsResponse => registered_models;
    fn create_registered_model(CreateRegisteredModelRequest) -> RegisteredModel;
    fn get_registered_model(GetRegisteredModelRequest) -> RegisteredModel;
    fn update_registered_model(UpdateRegisteredModelRequest) -> RegisteredModel;
    fn delete_registered_model(DeleteRegisteredModelRequest) -> ();
});

authorized!(SchemaHandler {
    list list_schemas(ListSchemasRequest) -> ListSchemasResponse => schemas;
    fn create_schema(CreateSchemaRequest) -> Schema;
    fn get_schema(GetSchemaRequest) -> Schema;
    fn update_schema(UpdateSchemaRequest) -> Schema;
    fn delete_schema(DeleteSchemaRequest) -> ();
});

authorized!(StagingTableHandler {
    fn create_staging_table(CreateStagingTableRequest) -> StagingTable;
});

authorized!(TableHandler {
    list list_table_summaries(ListTableSummariesRequest) -> ListTableSummariesResponse => tables by table_summary_ident;
    list list_tables(ListTablesRequest) -> ListTablesResponse => tables;
    fn create_table(CreateTableRequest) -> Table;
    fn get_table(GetTableRequest) -> Table;
    fn get_table_exists(GetTableExistsRequest) -> GetTableExistsResponse;
    fn delete_table(DeleteTableRequest) -> ();
});

authorized!(TagPolicyHandler {
    list list_tag_policies(ListTagPoliciesRequest) -> ListTagPoliciesResponse => tag_policies;
    fn create_tag_policy(CreateTagPolicyRequest) -> TagPolicy;
    fn get_tag_policy(GetTagPolicyRequest) -> TagPolicy;
    fn update_tag_policy(UpdateTagPolicyRequest) -> TagPolicy;
    fn delete_tag_policy(DeleteTagPolicyRequest) -> ();
});

authorized!(VolumeHandler {
    list list_volumes(ListVolumesRequest) -> ListVolumesResponse => volumes;
    fn create_volume(CreateVolumeRequest) -> Volume;
    fn get_volume(GetVolumeRequest) -> Volume;
    fn update_volume(UpdateVolumeRequest) -> Volume;
    fn delete_volume(DeleteVolumeRequest) -> ();
});

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use unitycatalog_common::models::resource_name;

    use super::*;
    use crate::Error;

    /// Denies the listed resources and allows everything else.
    struct DenyList(Vec<ResourceIdent>);

    #[async_trait]
    impl Policy<()> for DenyList {
        async fn authorize(
            &self,
            resource: &ResourceIdent,
            _permission: &Permission,
            _context: &(),
        ) -> Result<Decision> {
            Ok(if self.0.contains(resource) {
                Decision::Deny
            } else {
                Decision::Allow
            })
        }
    }

    fn deny(idents: impl IntoIterator<Item = ResourceIdent>) -> Arc<dyn Policy<()>> {
        Arc::new(DenyList(idents.into_iter().collect()))
    }

    fn catalog(name: &str) -> Catalog {
        Catalog {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Serves fixed catalogs, counting the calls that reach it.
    #[derive(Default)]
    struct Catalogs {
        calls: AtomicUsize,
    }

    impl Catalogs {
        fn called(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }

        fn hit(&self) {
            self.calls.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[async_trait]
    impl CatalogHandler<()> for Catalogs {
        async fn list_catalogs(
            &self,
            _request: ListCatalogsRequest,
            _context: (),
        ) -> Result<ListCatalogsResponse> {
            self.hit();
            Ok(ListCatalogsResponse {
                catalogs: ["a", "b", "c", "d"].map(catalog).into(),
                ..Default::default()
            })
        }

        async fn create_catalog(&self, request: CreateCatalogRequest, _: ()) -> Result<Catalog> {
            self.hit();
            Ok(catalog(&request.name))
        }

        async fn get_catalog(&self, request: GetCatalogRequest, _: ()) -> Result<Catalog> {
            self.hit();
            match request.name.as_str() {
                "gone" => Err(Error::NotFound),
                name => Ok(catalog(name)),
            }
        }

        async fn update_catalog(&self, request: UpdateCatalogRequest, _: ()) -> Result<Catalog> {
            self.hit();
            Ok(catalog(&request.name))
        }

        async fn delete_catalog(&self, _request: DeleteCatalogRequest, _: ()) -> Result<()> {
            self.hit();
            Ok(())
        }
    }

    fn get(name: &str) -> GetCatalogRequest {
        GetCatalogRequest {
            name: name.to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_denied_request_is_not_delegated() {
        let handler = Authorized::new(
            Catalogs::default(),
            deny([ResourceIdent::catalog(resource_name!("secret"))]),
        );

        let err = handler.get_catalog(get("secret"), ()).await.unwrap_err();
        assert!(matches!(err, Error::NotAllowed), "{err:?}");
        assert_eq!(handler.inner().called(), 0);

        let allowed = handler.get_catalog(get("main"), ()).await.unwrap();
        assert_eq!(allowed.name, "main");
        assert_eq!(handler.inner().called(), 1);
    }

    #[tokio::test]
    async fn test_inner_error_passes_through() {
        let handler = Authorized::new(Catalogs::default(), deny([]));
        let err = handler.get_catalog(get("gone"), ()).await.unwrap_err();
        assert!(matches!(err, Error::NotFound), "{err:?}");
    }

    #[tokio::test]
    async fn test_list_keeps_only_readable_resources_in_order() {
        let handler = Authorized::new(
            Catalogs::default(),
            deny([
                ResourceIdent::catalog(resource_name!("b")),
                ResourceIdent::catalog(resource_name!("d")),
            ]),
        );
        let resp = handler
            .list_catalogs(ListCatalogsRequest::default(), ())
            .await
            .unwrap();
        let names: Vec<_> = resp.catalogs.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["a", "c"]);
    }

    /// Serves fixed table summaries; every other method is unreachable here.
    struct Tables;

    #[async_trait]
    impl TableHandler<()> for Tables {
        async fn list_table_summaries(
            &self,
            _request: ListTableSummariesRequest,
            _context: (),
        ) -> Result<ListTableSummariesResponse> {
            let summary = |full_name: &str| TableSummary {
                full_name: full_name.to_string(),
                ..Default::default()
            };
            Ok(ListTableSummariesResponse {
                tables: vec![
                    summary("c.s.open"),
                    summary("c.s.hidden"),
                    summary("c.t.open"),
                ],
                ..Default::default()
            })
        }

        async fn list_tables(&self, _: ListTablesRequest, _: ()) -> Result<ListTablesResponse> {
            unreachable!()
        }

        async fn create_table(&self, _: CreateTableRequest, _: ()) -> Result<Table> {
            unreachable!()
        }

        async fn get_table(&self, _: GetTableRequest, _: ()) -> Result<Table> {
            unreachable!()
        }

        async fn get_table_exists(
            &self,
            _: GetTableExistsRequest,
            _: (),
        ) -> Result<GetTableExistsResponse> {
            unreachable!()
        }

        async fn delete_table(&self, _: DeleteTableRequest, _: ()) -> Result<()> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn test_table_summaries_filtered_by_full_name() {
        let handler = Authorized::new(
            Tables,
            deny([ResourceIdent::table(resource_name!("c.s.hidden"))]),
        );
        let request = ListTableSummariesRequest {
            catalog_name: "c".to_string(),
            ..Default::default()
        };
        let resp = handler.list_table_summaries(request, ()).await.unwrap();
        let names: Vec<_> = resp.tables.iter().map(|t| t.full_name.as_str()).collect();
        assert_eq!(names, ["c.s.open", "c.t.open"]);
    }

    /// An upstream non-2xx reply surfaces as the matching server error, so the
    /// proxy answers `404` for a missing upstream catalog rather than `500`.
    #[cfg(feature = "proxy")]
    #[tokio::test]
    async fn test_upstream_not_found_maps_to_not_found() {
        use unitycatalog_client::UnityCatalogClient;

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/catalogs/gone")
            .with_status(404)
            .with_body(r#"{"error_code":"CATALOG_DOES_NOT_EXIST","message":"no catalog"}"#)
            .create_async()
            .await;
        let client = UnityCatalogClient::new_unauthenticated(server.url().parse().unwrap())
            .catalogs_client();

        let handler = Authorized::new(client, deny([]));
        let err = handler.get_catalog(get("gone"), ()).await.unwrap_err();
        assert!(matches!(err, Error::NotFound), "{err:?}");
        mock.assert_async().await;
    }
}
