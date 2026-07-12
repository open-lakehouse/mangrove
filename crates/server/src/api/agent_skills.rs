use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::staging_tables::{child_location, resolve_managed_parent_location};
use super::{RequestContext, SecuredAction};
pub use crate::codegen::agent_skills::AgentSkillHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::services::location::StorageLocationUrl;
use crate::services::object_store::validate_external_storage_location;
use crate::services::{ProvidesLocalStoragePolicy, ProvidesManagedStorageRoot};
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
impl<
    T: ResourceStore
        + Policy<RequestContext>
        + ProvidesLocalStoragePolicy
        + ProvidesManagedStorageRoot,
> AgentSkillHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_agent_skill(
        &self,
        request: CreateAgentSkillRequest,
        context: RequestContext,
    ) -> Result<AgentSkill> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;

        let skill_type = AgentSkillType::try_from(request.agent_skill_type)
            .unwrap_or(AgentSkillType::Unspecified);

        // Pre-allocate the id so the created record carries it (the store honors a
        // pre-set id, else mints a v7) and a managed skill can embed it in its
        // storage path. This mirrors managed `Volume`, since an agent skill is
        // just a storage-backed directory.
        let agent_skill_id = uuid::Uuid::now_v7().hyphenated().to_string();
        let storage_location = match skill_type {
            AgentSkillType::External => {
                // External skills MUST have an explicit storage location that
                // lives within a registered external location and does not
                // overlap any existing table or volume.
                let location = request
                    .storage_location
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        Error::invalid_argument(
                            "storage_location is required for EXTERNAL agent skills",
                        )
                    })?;
                let parsed = StorageLocationUrl::parse(&location)?;
                validate_external_storage_location(self, &parsed).await?;
                location
            }
            AgentSkillType::Managed => {
                // Managed skills derive their storage location from the managed
                // parent location resolved for the schema/catalog, appending an
                // `agent_skills/{id}` segment, mirroring managed volumes. The id
                // is allocated here and persisted so the path equals the skill's
                // id and survives renames. A caller cannot supply a location.
                let parent = resolve_managed_parent_location(
                    self,
                    &request.catalog_name,
                    &request.schema_name,
                )
                .await?;
                child_location(&parent, "agent_skills", &agent_skill_id)
            }
            AgentSkillType::Unspecified => {
                return Err(Error::invalid_argument(
                    "agent_skill_type must be specified (EXTERNAL or MANAGED)",
                ));
            }
        };

        let full_name = format!(
            "{}.{}.{}",
            request.catalog_name, request.schema_name, request.name
        );
        let resource = AgentSkill {
            full_name,
            name: request.name,
            catalog_name: request.catalog_name,
            schema_name: request.schema_name,
            agent_skill_type: request.agent_skill_type,
            storage_location,
            agent_skill_id,
            description: request.description,
            license: request.license,
            allowed_tools: request.allowed_tools,
            metadata: request.metadata,
            comment: request.comment,
            ..Default::default()
        };
        Ok(self.create(resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_agent_skills(
        &self,
        request: ListAgentSkillsRequest,
        context: RequestContext,
    ) -> Result<ListAgentSkillsResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::AgentSkill,
                Some(&ResourceName::new([
                    &request.catalog_name,
                    &request.schema_name,
                ])),
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListAgentSkillsResponse {
            agent_skills: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_agent_skill(
        &self,
        request: GetAgentSkillRequest,
        context: RequestContext,
    ) -> Result<AgentSkill> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_agent_skill(
        &self,
        request: UpdateAgentSkillRequest,
        context: RequestContext,
    ) -> Result<AgentSkill> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let name = ResourceName::from_naive_str_split(request.name.as_str());
        let [catalog_name, schema_name, skill_name] = name.as_ref() else {
            return Err(Error::invalid_argument(
                "Invalid agent skill name - expected <catalog_name>.<schema_name>.<skill_name>",
            ));
        };
        // Load the current record so omitted update fields (and the immutable
        // storage location / type / id) are preserved across the rename-style update.
        let current: AgentSkill = self.get(&ident).await?.0.try_into()?;
        let new_name = request.new_name.as_deref().unwrap_or(skill_name);
        let resource = AgentSkill {
            name: new_name.to_owned(),
            catalog_name: catalog_name.to_owned(),
            schema_name: schema_name.to_owned(),
            full_name: format!("{}.{}.{}", catalog_name, schema_name, new_name),
            agent_skill_type: current.agent_skill_type,
            storage_location: current.storage_location,
            description: request.description.or(current.description),
            license: current.license,
            allowed_tools: if request.allowed_tools.is_empty() {
                current.allowed_tools
            } else {
                request.allowed_tools
            },
            metadata: current.metadata,
            comment: request.comment.or(current.comment),
            owner: request.owner.or(current.owner),
            ..Default::default()
        };
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_agent_skill(
        &self,
        request: DeleteAgentSkillRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }
}

impl SecuredAction for CreateAgentSkillRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::agent_skill(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
            self.name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListAgentSkillsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::agent_skill(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetAgentSkillRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::agent_skill(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateAgentSkillRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::agent_skill(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteAgentSkillRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::agent_skill(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::catalogs::v1::CreateCatalogRequest;
    use unitycatalog_common::models::credentials::v1::{
        AwsIamRoleConfig, CreateCredentialRequest, Purpose,
    };
    use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
    use unitycatalog_common::models::schemas::v1::CreateSchemaRequest;
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::api::{CatalogHandler, CredentialHandler, ExternalLocationHandler, SchemaHandler};
    use crate::memory::InMemoryResourceStore;
    use crate::policy::ConstantPolicy;
    use crate::services::ServerHandler;

    fn handler() -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        ServerHandler::try_new_tokio(policy, store).unwrap()
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: crate::policy::Principal::anonymous(),
        }
    }

    /// Register a credential + external location at `url`.
    async fn make_covering_location(h: &ServerHandler<RequestContext>, tag: &str, url: &str) {
        h.create_credential(
            CreateCredentialRequest {
                name: format!("{tag}-cred"),
                purpose: Purpose::Storage as i32,
                aws_iam_role: Some(AwsIamRoleConfig {
                    role_arn: "arn:aws:iam::123456789012:role/test".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_external_location(
            CreateExternalLocationRequest {
                name: format!("{tag}-el"),
                url: url.to_string(),
                credential_name: format!("{tag}-cred"),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    /// Create catalog `cat` (rooted at `storage_root`) and schema `sch` so a
    /// managed skill can resolve a managed storage root.
    async fn setup_managed_namespace(h: &ServerHandler<RequestContext>, storage_root: &str) {
        make_covering_location(h, "cat", storage_root).await;
        h.create_catalog(
            CreateCatalogRequest {
                name: "cat".to_string(),
                storage_root: Some(storage_root.to_string()),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_schema(
            CreateSchemaRequest {
                name: "sch".to_string(),
                catalog_name: "cat".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    fn create_external_skill(name: &str, location: Option<&str>) -> CreateAgentSkillRequest {
        CreateAgentSkillRequest {
            catalog_name: "cat".to_string(),
            schema_name: "sch".to_string(),
            name: name.to_string(),
            agent_skill_type: AgentSkillType::External as i32,
            storage_location: location.map(str::to_string),
            description: Some("formats things".to_string()),
            license: Some("MIT".to_string()),
            allowed_tools: vec!["bash".to_string()],
            metadata: Default::default(),
            comment: None,
        }
    }

    fn create_managed_skill(name: &str) -> CreateAgentSkillRequest {
        CreateAgentSkillRequest {
            catalog_name: "cat".to_string(),
            schema_name: "sch".to_string(),
            name: name.to_string(),
            agent_skill_type: AgentSkillType::Managed as i32,
            storage_location: None,
            description: None,
            license: None,
            allowed_tools: vec![],
            metadata: Default::default(),
            comment: None,
        }
    }

    #[tokio::test]
    async fn managed_skill_location_uses_skill_id() {
        let h = handler();
        setup_managed_namespace(&h, "s3://bucket/cat").await;

        let s = h
            .create_agent_skill(create_managed_skill("fmt"), ctx())
            .await
            .unwrap();

        assert!(uuid::Uuid::parse_str(&s.agent_skill_id).is_ok());
        assert!(
            s.storage_location
                .ends_with(&format!("/agent_skills/{}", s.agent_skill_id)),
            "got {}",
            s.storage_location
        );
        assert!(
            s.storage_location
                .starts_with("s3://bucket/cat/__unitystorage/catalogs/"),
            "got {}",
            s.storage_location
        );

        let got = h
            .get_agent_skill(
                GetAgentSkillRequest {
                    name: "cat.sch.fmt".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(got.agent_skill_id, s.agent_skill_id);
        assert_eq!(got.storage_location, s.storage_location);
    }

    #[tokio::test]
    async fn managed_skill_recreate_yields_new_path() {
        let h = handler();
        setup_managed_namespace(&h, "s3://bucket/cat").await;
        let first = h
            .create_agent_skill(create_managed_skill("fmt"), ctx())
            .await
            .unwrap();
        h.delete_agent_skill(
            DeleteAgentSkillRequest {
                name: "cat.sch.fmt".to_string(),
            },
            ctx(),
        )
        .await
        .unwrap();
        let second = h
            .create_agent_skill(create_managed_skill("fmt"), ctx())
            .await
            .unwrap();
        assert_ne!(first.agent_skill_id, second.agent_skill_id);
        assert_ne!(first.storage_location, second.storage_location);
    }

    #[tokio::test]
    async fn external_skill_within_location_succeeds_and_round_trips_metadata() {
        let h = handler();
        // Managed namespace (catalog/schema) plus a separate external location
        // covering the skill's own path.
        setup_managed_namespace(&h, "s3://bucket/cat").await;
        make_covering_location(&h, "ext", "s3://bucket/ext").await;

        let created = h
            .create_agent_skill(
                create_external_skill("fmt", Some("s3://bucket/ext/skill")),
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(created.storage_location, "s3://bucket/ext/skill");
        assert_eq!(created.license.as_deref(), Some("MIT"));
        assert_eq!(created.allowed_tools, vec!["bash".to_string()]);
        assert_eq!(created.description.as_deref(), Some("formats things"));
    }

    #[tokio::test]
    async fn external_skill_outside_location_is_rejected() {
        let h = handler();
        setup_managed_namespace(&h, "s3://bucket/cat").await;
        let res = h
            .create_agent_skill(
                create_external_skill("fmt", Some("s3://bucket/other/skill")),
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn external_skill_requires_storage_location() {
        let h = handler();
        setup_managed_namespace(&h, "s3://bucket/cat").await;
        let res = h
            .create_agent_skill(create_external_skill("fmt", None), ctx())
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn managed_skill_duplicate_name_is_rejected() {
        let h = handler();
        setup_managed_namespace(&h, "s3://bucket/cat").await;
        h.create_agent_skill(create_managed_skill("fmt"), ctx())
            .await
            .unwrap();
        let err = h
            .create_agent_skill(create_managed_skill("fmt"), ctx())
            .await
            .expect_err("duplicate name must be rejected");
        assert_eq!(err.error_code(), "RESOURCE_ALREADY_EXISTS", "{err:?}");
    }
}
