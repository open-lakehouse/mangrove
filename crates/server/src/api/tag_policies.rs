use itertools::Itertools;

use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::{ObjectLabel, ResourceIdent, ResourceName, ResourceRef};

use super::{RequestContext, SecuredAction, UpdateMask};
use crate::Result;
pub use crate::codegen::tag_policies::TagPolicyHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::store::ResourceStore;

/// The largest page ListTagPolicies returns.
const MAX_PAGE_SIZE: usize = 1000;

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> TagPolicyHandler<RequestContext> for T {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_tag_policy(
        &self,
        request: CreateTagPolicyRequest,
        context: RequestContext,
    ) -> Result<TagPolicy> {
        self.check_required(&request, &context).await?;
        tracing::Span::current().record("resource_name", &request.tag_key);
        if request.tag_key.is_empty() {
            return Err(crate::Error::invalid_argument("tag_key must be provided"));
        }
        let resource = TagPolicy {
            tag_key: request.tag_key,
            description: request.description,
            values: request.values,
            ..Default::default()
        };
        Ok(self.create(resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_tag_policy(
        &self,
        request: DeleteTagPolicyRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.tag_key);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_tag_policy(
        &self,
        request: GetTagPolicyRequest,
        context: RequestContext,
    ) -> Result<TagPolicy> {
        tracing::Span::current().record("resource_name", &request.tag_key);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_tag_policies(
        &self,
        request: ListTagPoliciesRequest,
        context: RequestContext,
    ) -> Result<ListTagPoliciesResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::TagPolicy,
                None,
                // Unset or 0 means the 1000 default; larger values are coerced down.
                Some(
                    super::page_size(request.max_results)?
                        .map_or(MAX_PAGE_SIZE, |n| n.min(MAX_PAGE_SIZE)),
                ),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListTagPoliciesResponse {
            tag_policies: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_tag_policy(
        &self,
        request: UpdateTagPolicyRequest,
        context: RequestContext,
    ) -> Result<TagPolicy> {
        tracing::Span::current().record("resource_name", &request.tag_key);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let mask = UpdateMask::parse(request.update_mask.as_deref(), &["description", "values"])?;
        let mut resource: TagPolicy = self.get(&ident).await?.0.try_into()?;
        match mask {
            UpdateMask::All => {
                if request.description.is_some() {
                    resource.description = request.description;
                }
                if !request.values.is_empty() {
                    resource.values = request.values;
                }
            }
            UpdateMask::Fields(_) => {
                if mask.names("description") {
                    resource.description = request.description;
                }
                if mask.names("values") {
                    resource.values = request.values;
                }
            }
        }
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }
}

impl SecuredAction for CreateTagPolicyRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::tag_policy(ResourceName::new([self.tag_key.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListTagPoliciesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::tag_policy(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetTagPolicyRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::tag_policy(ResourceName::new([self.tag_key.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateTagPolicyRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::tag_policy(ResourceName::new([self.tag_key.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteTagPolicyRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::tag_policy(ResourceName::new([self.tag_key.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
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

    fn policy(tag_key: &str) -> TagPolicy {
        TagPolicy {
            tag_key: tag_key.to_string(),
            description: Some("classification".to_string()),
            values: vec![
                Value {
                    name: "public".to_string(),
                    ..Default::default()
                },
                Value {
                    name: "restricted".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    fn create_request(policy: TagPolicy) -> CreateTagPolicyRequest {
        CreateTagPolicyRequest {
            tag_key: policy.tag_key,
            description: policy.description,
            values: policy.values,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn tag_policy_crud_round_trip() {
        let h = handler();

        // create
        let created = h
            .create_tag_policy(create_request(policy("classification")), ctx())
            .await
            .unwrap();
        assert_eq!(created.tag_key, "classification");
        assert_eq!(created.values.len(), 2);

        // get
        let fetched = h
            .get_tag_policy(
                GetTagPolicyRequest {
                    tag_key: "classification".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(fetched.tag_key, "classification");
        assert_eq!(fetched.description.as_deref(), Some("classification"));

        // list
        let listed = h
            .list_tag_policies(ListTagPoliciesRequest::default(), ctx())
            .await
            .unwrap();
        assert_eq!(listed.tag_policies.len(), 1);
        assert_eq!(listed.tag_policies[0].tag_key, "classification");

        // update — change description and allowed values
        let updated = h
            .update_tag_policy(
                UpdateTagPolicyRequest {
                    tag_key: "classification".to_string(),
                    description: Some("updated".to_string()),
                    values: vec![Value {
                        name: "public".to_string(),
                        ..Default::default()
                    }],
                    update_mask: None,
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(updated.description.as_deref(), Some("updated"));
        assert_eq!(updated.values.len(), 1);

        // delete
        h.delete_tag_policy(
            DeleteTagPolicyRequest {
                tag_key: "classification".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();

        // get after delete → not found
        let missing = h
            .get_tag_policy(
                GetTagPolicyRequest {
                    tag_key: "classification".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(missing.is_err(), "expected NotFound after delete");
    }

    #[tokio::test]
    async fn create_without_tag_key_is_invalid() {
        let h = handler();
        let result = h
            .create_tag_policy(CreateTagPolicyRequest::default(), ctx())
            .await;
        assert!(result.is_err(), "missing tag_key must be rejected");
    }

    #[tokio::test]
    async fn update_mask_clears_listed_fields_only() {
        let h = handler();
        h.create_tag_policy(create_request(policy("classification")), ctx())
            .await
            .unwrap();
        let updated = h
            .update_tag_policy(
                UpdateTagPolicyRequest {
                    tag_key: "classification".to_string(),
                    update_mask: Some("description".to_string()),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(updated.description, None);
        assert_eq!(updated.values.len(), 2, "values are not in the mask");
    }
}
