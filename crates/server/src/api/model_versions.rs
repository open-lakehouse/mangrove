use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::model_versions::v1::*;
use unitycatalog_common::models::registered_models::v1::RegisteredModel;
use unitycatalog_common::models::{ResourceIdent, ResourceName};

use super::{RegisteredModelHandler, RequestContext, SecuredAction};
pub use crate::codegen::model_versions::ModelVersionHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::store::ResourceStore;
use crate::{Error, Result};
use unitycatalog_common::models::registered_models::v1::GetRegisteredModelRequest;

/// Split a registered model's three-level `full_name` into its components.
fn split_model_full_name(full_name: &str) -> Result<(String, String, String)> {
    let name = ResourceName::from_naive_str_split(full_name);
    let [catalog, schema, model] = name.as_ref() else {
        return Err(Error::invalid_argument(
            "Invalid model name - expected <catalog_name>.<schema_name>.<model_name>",
        ));
    };
    Ok((catalog.clone(), schema.clone(), model.clone()))
}

/// The store `ResourceIdent` for a specific model version, keyed by the composite
/// `[catalog, schema, model_name, version]` name — matching `ModelVersion::resource_name`.
fn model_version_ident(catalog: &str, schema: &str, model: &str, version: i64) -> ResourceIdent {
    ResourceIdent::model_version(ResourceName::new([
        catalog.to_string(),
        schema.to_string(),
        model.to_string(),
        version.to_string(),
    ]))
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext> + RegisteredModelHandler<RequestContext>>
    ModelVersionHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn list_model_versions(
        &self,
        request: ListModelVersionsRequest,
        context: RequestContext,
    ) -> Result<ListModelVersionsResponse> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        let (catalog, schema, model) = split_model_full_name(&request.full_name)?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::ModelVersion,
                Some(&ResourceName::new([&catalog, &schema, &model])),
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListModelVersionsResponse {
            model_versions: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_model_version(
        &self,
        request: CreateModelVersionRequest,
        context: RequestContext,
    ) -> Result<ModelVersion> {
        self.check_required(&request, &context).await?;
        let full_name = format!(
            "{}.{}.{}",
            request.catalog_name, request.schema_name, request.model_name
        );
        tracing::Span::current().record("resource_name", &full_name);

        // Resolve the parent registered model to derive the version's storage location
        // under the model's managed root, and to confirm the model exists.
        let parent = self
            .get_registered_model(
                GetRegisteredModelRequest {
                    full_name: full_name.clone(),
                    ..Default::default()
                },
                context.clone(),
            )
            .await?;

        // Assign the next version number. Versions are monotonically increasing per
        // model; a fresh model starts at version 1.
        let next_version = self.next_model_version(&parent).await?;

        let storage_location = parent
            .storage_location
            .as_ref()
            .map(|root| format!("{root}/versions/{next_version}"));

        let id = uuid::Uuid::now_v7().hyphenated().to_string();
        let resource = ModelVersion {
            model_name: request.model_name,
            catalog_name: request.catalog_name,
            schema_name: request.schema_name,
            version: next_version,
            source: request.source,
            run_id: request.run_id,
            status: ModelVersionStatus::PendingRegistration.into(),
            storage_location,
            comment: request.comment,
            id: Some(id),
            ..Default::default()
        };
        Ok(self.create(resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_model_version(
        &self,
        request: GetModelVersionRequest,
        context: RequestContext,
    ) -> Result<ModelVersion> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_model_version(
        &self,
        request: UpdateModelVersionRequest,
        context: RequestContext,
    ) -> Result<ModelVersion> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        // Only `comment` is mutable via update; preserve the rest by reading first.
        let (current, _) = self.get(&ident).await?;
        let mut model_version: ModelVersion = current.try_into()?;
        model_version.comment = request.comment;
        Ok(self
            .update(&ident, model_version.into())
            .await?
            .0
            .try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_model_version(
        &self,
        request: DeleteModelVersionRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn finalize_model_version(
        &self,
        request: FinalizeModelVersionRequest,
        context: RequestContext,
    ) -> Result<ModelVersion> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let (current, _) = self.get(&ident).await?;
        let mut model_version: ModelVersion = current.try_into()?;
        // Finalization transitions the version to READY, signaling that all
        // artifacts have been written to its storage location.
        model_version.status = ModelVersionStatus::Ready.into();
        Ok(self
            .update(&ident, model_version.into())
            .await?
            .0
            .try_into()?)
    }
}

/// Version-allocation helper, factored out so it is not part of the public handler trait.
#[async_trait::async_trait]
trait ModelVersionAllocExt: ResourceStore {
    /// Compute the next version number for a registered model: one greater than the
    /// highest existing version, or 1 for a model with no versions yet.
    async fn next_model_version(&self, parent: &RegisteredModel) -> Result<i64> {
        let namespace =
            ResourceName::new([&parent.catalog_name, &parent.schema_name, &parent.name]);
        // Page through all existing versions to find the current maximum. Model
        // version counts are small, so a full scan is acceptable.
        let mut max_version = 0i64;
        let mut page_token = None;
        loop {
            let (resources, next) = self
                .list(
                    &ObjectLabel::ModelVersion,
                    Some(&namespace),
                    None,
                    page_token,
                )
                .await?;
            for resource in resources {
                let mv: ModelVersion = resource.try_into()?;
                max_version = max_version.max(mv.version);
            }
            match next {
                Some(token) => page_token = Some(token),
                None => break,
            }
        }
        Ok(max_version + 1)
    }
}

impl<T: ResourceStore> ModelVersionAllocExt for T {}

impl SecuredAction for CreateModelVersionRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::registered_model(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
            self.model_name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListModelVersionsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::registered_model(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetModelVersionRequest {
    fn resource(&self) -> ResourceIdent {
        match split_model_full_name(&self.full_name) {
            Ok((catalog, schema, model)) => {
                model_version_ident(&catalog, &schema, &model, self.version)
            }
            // A malformed name yields an undefined ref; the handler's name parse
            // surfaces the real error.
            Err(_) => ResourceIdent::model_version(ResourceName::from_naive_str_split(
                self.full_name.as_str(),
            )),
        }
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateModelVersionRequest {
    fn resource(&self) -> ResourceIdent {
        match split_model_full_name(&self.full_name) {
            Ok((catalog, schema, model)) => {
                model_version_ident(&catalog, &schema, &model, self.version)
            }
            Err(_) => ResourceIdent::model_version(ResourceName::from_naive_str_split(
                self.full_name.as_str(),
            )),
        }
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteModelVersionRequest {
    fn resource(&self) -> ResourceIdent {
        match split_model_full_name(&self.full_name) {
            Ok((catalog, schema, model)) => {
                model_version_ident(&catalog, &schema, &model, self.version)
            }
            Err(_) => ResourceIdent::model_version(ResourceName::from_naive_str_split(
                self.full_name.as_str(),
            )),
        }
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for FinalizeModelVersionRequest {
    fn resource(&self) -> ResourceIdent {
        match split_model_full_name(&self.full_name) {
            Ok((catalog, schema, model)) => {
                model_version_ident(&catalog, &schema, &model, self.version)
            }
            Err(_) => ResourceIdent::model_version(ResourceName::from_naive_str_split(
                self.full_name.as_str(),
            )),
        }
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
    use unitycatalog_common::models::registered_models::v1::CreateRegisteredModelRequest;
    use unitycatalog_common::models::schemas::v1::CreateSchemaRequest;
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::api::{
        CatalogHandler, CredentialHandler, ExternalLocationHandler, RegisteredModelHandler,
        SchemaHandler,
    };
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

    /// Register a covering external location + credential for `url`.
    async fn make_covering_location(h: &ServerHandler<RequestContext>, tag: &str, url: &str) {
        h.create_credential(
            CreateCredentialRequest {
                name: format!("{tag}-cred"),
                purpose: Purpose::Storage.into(),
                aws_iam_role: Some(AwsIamRoleConfig {
                    role_arn: "arn:aws:iam::123456789012:role/test".to_string(),
                    ..Default::default()
                })
                .into(),
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

    /// Create catalog `cat` (rooted at `storage_root`), schema `sch`, and registered
    /// model `mdl` so model versions can be created beneath it.
    async fn setup_model(h: &ServerHandler<RequestContext>, storage_root: &str) {
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
        h.create_registered_model(
            CreateRegisteredModelRequest {
                name: "mdl".to_string(),
                catalog_name: "cat".to_string(),
                schema_name: "sch".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    fn create_version(source: &str) -> CreateModelVersionRequest {
        CreateModelVersionRequest {
            model_name: "mdl".to_string(),
            catalog_name: "cat".to_string(),
            schema_name: "sch".to_string(),
            source: source.to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn create_assigns_monotonic_versions_and_pending_status() {
        let h = handler();
        setup_model(&h, "s3://bucket/cat").await;

        let v1 = h
            .create_model_version(create_version("s3://src/1"), ctx())
            .await
            .unwrap();
        assert_eq!(v1.version, 1);
        assert_eq!(
            v1.status.as_known(),
            Some(ModelVersionStatus::PendingRegistration)
        );
        // The version's storage location nests under the model's managed root.
        let loc = v1.storage_location.as_deref().unwrap();
        assert!(loc.ends_with("/versions/1"), "got {loc}");
        assert!(
            loc.contains("/__unitystorage/catalogs/") && loc.contains("/models/"),
            "got {loc}"
        );

        let v2 = h
            .create_model_version(create_version("s3://src/2"), ctx())
            .await
            .unwrap();
        assert_eq!(v2.version, 2);
    }

    #[tokio::test]
    async fn get_and_finalize_transitions_to_ready() {
        let h = handler();
        setup_model(&h, "s3://bucket/cat").await;
        h.create_model_version(create_version("s3://src/1"), ctx())
            .await
            .unwrap();

        let got = h
            .get_model_version(
                GetModelVersionRequest {
                    full_name: "cat.sch.mdl".to_string(),
                    version: 1,
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(got.version, 1);
        assert_eq!(
            got.status.as_known(),
            Some(ModelVersionStatus::PendingRegistration)
        );

        let finalized = h
            .finalize_model_version(
                FinalizeModelVersionRequest {
                    full_name: "cat.sch.mdl".to_string(),
                    version: 1,
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(finalized.status.as_known(), Some(ModelVersionStatus::Ready));
    }

    #[tokio::test]
    async fn list_returns_versions_for_the_model() {
        let h = handler();
        setup_model(&h, "s3://bucket/cat").await;
        h.create_model_version(create_version("s3://src/1"), ctx())
            .await
            .unwrap();
        h.create_model_version(create_version("s3://src/2"), ctx())
            .await
            .unwrap();

        let listed = h
            .list_model_versions(
                ListModelVersionsRequest {
                    full_name: "cat.sch.mdl".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        let mut versions: Vec<i64> = listed.model_versions.iter().map(|v| v.version).collect();
        versions.sort_unstable();
        assert_eq!(versions, vec![1, 2]);
    }

    #[tokio::test]
    async fn delete_removes_the_version() {
        let h = handler();
        setup_model(&h, "s3://bucket/cat").await;
        h.create_model_version(create_version("s3://src/1"), ctx())
            .await
            .unwrap();
        h.delete_model_version(
            DeleteModelVersionRequest {
                full_name: "cat.sch.mdl".to_string(),
                version: 1,
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        let res = h
            .get_model_version(
                GetModelVersionRequest {
                    full_name: "cat.sch.mdl".to_string(),
                    version: 1,
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(res.is_err(), "deleted version should not be found");
    }
}
