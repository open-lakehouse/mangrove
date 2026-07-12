use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::agents::v0alpha1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::agents::AgentHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> AgentHandler<RequestContext> for T {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_agent(
        &self,
        request: CreateAgentRequest,
        context: RequestContext,
    ) -> Result<Agent> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;

        // Agents are metadata-only: validate the invocation protocol is a known
        // variant (a 2-arg `try_from` rejects unspecified/unknown values) and
        // persist the record. There is no storage location to resolve.
        let protocol = InvocationProtocol::try_from(request.invocation_protocol)
            .ok()
            .filter(|p| *p != InvocationProtocol::Unspecified)
            .ok_or_else(|| {
                Error::invalid_argument("invocation_protocol must be a known protocol")
            })?;

        let full_name = format!(
            "{}.{}.{}",
            request.catalog_name, request.schema_name, request.name
        );
        // Pre-allocate the id so the created record (and its response) carries it;
        // the store honors a pre-set id (else it mints a v7), matching catalogs.
        let resource = Agent {
            full_name,
            name: request.name,
            catalog_name: request.catalog_name,
            schema_name: request.schema_name,
            agent_id: uuid::Uuid::now_v7().hyphenated().to_string(),
            invocation_protocol: protocol as i32,
            endpoint: request.endpoint,
            description: request.description,
            capabilities: request.capabilities,
            input_schema: request.input_schema,
            comment: request.comment,
            ..Default::default()
        };
        Ok(self.create(resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_agents(
        &self,
        request: ListAgentsRequest,
        context: RequestContext,
    ) -> Result<ListAgentsResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Agent,
                Some(&ResourceName::new([
                    &request.catalog_name,
                    &request.schema_name,
                ])),
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListAgentsResponse {
            agents: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_agent(&self, request: GetAgentRequest, context: RequestContext) -> Result<Agent> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_agent(
        &self,
        request: UpdateAgentRequest,
        context: RequestContext,
    ) -> Result<Agent> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let name = ResourceName::from_naive_str_split(request.name.as_str());
        let [catalog_name, schema_name, agent_name] = name.as_ref() else {
            return Err(Error::invalid_argument(
                "Invalid agent name - expected <catalog_name>.<schema_name>.<agent_name>",
            ));
        };
        // Load the current record so omitted update fields keep their value.
        let current: Agent = self.get(&ident).await?.0.try_into()?;
        let new_name = request.new_name.as_deref().unwrap_or(agent_name);
        let protocol = match request.invocation_protocol {
            Some(p) => InvocationProtocol::try_from(p)
                .ok()
                .filter(|p| *p != InvocationProtocol::Unspecified)
                .ok_or_else(|| {
                    Error::invalid_argument("invocation_protocol must be a known protocol")
                })? as i32,
            None => current.invocation_protocol,
        };
        let resource = Agent {
            name: new_name.to_owned(),
            catalog_name: catalog_name.to_owned(),
            schema_name: schema_name.to_owned(),
            full_name: format!("{}.{}.{}", catalog_name, schema_name, new_name),
            invocation_protocol: protocol,
            endpoint: request.endpoint.unwrap_or(current.endpoint),
            description: request.description.or(current.description),
            capabilities: if request.capabilities.is_empty() {
                current.capabilities
            } else {
                request.capabilities
            },
            input_schema: request.input_schema.or(current.input_schema),
            comment: request.comment.or(current.comment),
            owner: request.owner.or(current.owner),
            ..Default::default()
        };
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_agent(
        &self,
        request: DeleteAgentRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }
}

impl SecuredAction for CreateAgentRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::agent(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
            self.name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListAgentsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::agent(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetAgentRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::agent(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateAgentRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::agent(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteAgentRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::agent(ResourceName::from_naive_str_split(self.name.as_str()))
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

    /// Create catalog `cat` (rooted at a covered storage root) and schema `sch`
    /// so agents can be created beneath them. Agents are metadata-only, but the
    /// parent managed catalog still requires a resolvable storage root.
    async fn setup_namespace(h: &ServerHandler<RequestContext>) {
        h.create_credential(
            CreateCredentialRequest {
                name: "cred".to_string(),
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
                name: "el".to_string(),
                url: "s3://bucket/cat".to_string(),
                credential_name: "cred".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_catalog(
            CreateCatalogRequest {
                name: "cat".to_string(),
                storage_root: Some("s3://bucket/cat".to_string()),
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

    fn create_request(name: &str, protocol: InvocationProtocol) -> CreateAgentRequest {
        CreateAgentRequest {
            catalog_name: "cat".to_string(),
            schema_name: "sch".to_string(),
            name: name.to_string(),
            invocation_protocol: protocol as i32,
            endpoint: "https://agent.example.com".to_string(),
            description: Some("does things".to_string()),
            capabilities: vec!["sql_query".to_string()],
            input_schema: Some("{\"type\":\"object\"}".to_string()),
            comment: None,
        }
    }

    #[tokio::test]
    async fn create_get_round_trip() {
        let h = handler();
        setup_namespace(&h).await;

        let created = h
            .create_agent(create_request("agt", InvocationProtocol::Mcp), ctx())
            .await
            .unwrap();
        assert_eq!(created.full_name, "cat.sch.agt");
        assert_eq!(created.invocation_protocol, InvocationProtocol::Mcp as i32);
        assert_eq!(created.endpoint, "https://agent.example.com");
        assert_eq!(created.capabilities, vec!["sql_query".to_string()]);
        assert!(uuid::Uuid::parse_str(&created.agent_id).is_ok());

        let got = h
            .get_agent(
                GetAgentRequest {
                    name: "cat.sch.agt".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(got.agent_id, created.agent_id);
        assert_eq!(got.input_schema.as_deref(), Some("{\"type\":\"object\"}"));
    }

    #[tokio::test]
    async fn unspecified_protocol_is_rejected() {
        let h = handler();
        setup_namespace(&h).await;
        let res = h
            .create_agent(
                create_request("agt", InvocationProtocol::Unspecified),
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn list_and_delete() {
        let h = handler();
        setup_namespace(&h).await;
        h.create_agent(create_request("a1", InvocationProtocol::Rest), ctx())
            .await
            .unwrap();
        h.create_agent(create_request("a2", InvocationProtocol::Anthropic), ctx())
            .await
            .unwrap();

        let listed = h
            .list_agents(
                ListAgentsRequest {
                    catalog_name: "cat".to_string(),
                    schema_name: "sch".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(listed.agents.len(), 2);

        h.delete_agent(
            DeleteAgentRequest {
                name: "cat.sch.a1".to_string(),
            },
            ctx(),
        )
        .await
        .unwrap();
        let listed = h
            .list_agents(
                ListAgentsRequest {
                    catalog_name: "cat".to_string(),
                    schema_name: "sch".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(listed.agents.len(), 1);
    }

    #[tokio::test]
    async fn update_preserves_omitted_fields() {
        let h = handler();
        setup_namespace(&h).await;
        h.create_agent(create_request("agt", InvocationProtocol::Mcp), ctx())
            .await
            .unwrap();

        // Update only the endpoint; protocol/description/capabilities are preserved.
        let updated = h
            .update_agent(
                UpdateAgentRequest {
                    name: "cat.sch.agt".to_string(),
                    endpoint: Some("https://new.example.com".to_string()),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(updated.endpoint, "https://new.example.com");
        assert_eq!(updated.invocation_protocol, InvocationProtocol::Mcp as i32);
        assert_eq!(updated.capabilities, vec!["sql_query".to_string()]);
        assert_eq!(updated.description.as_deref(), Some("does things"));
    }

    #[tokio::test]
    async fn duplicate_name_is_rejected() {
        let h = handler();
        setup_namespace(&h).await;
        h.create_agent(create_request("agt", InvocationProtocol::Mcp), ctx())
            .await
            .unwrap();
        let err = h
            .create_agent(create_request("agt", InvocationProtocol::Mcp), ctx())
            .await
            .expect_err("duplicate name must be rejected");
        assert_eq!(err.error_code(), "RESOURCE_ALREADY_EXISTS", "{err:?}");
    }
}
