use unitycatalog_common::models::external_locations::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};
use unitycatalog_storage_access::{ExternalLocationUpdate, Page, ProvidesStorageAccessStore};

use super::{RequestContext, SecuredAction};
use crate::error::from_store;
use crate::policy::{Permission, Policy, process_resources};
use crate::services::ProvidesLocalStoragePolicy;
use crate::services::ServerHandler;
use crate::services::location::StorageLocationUrl;
use crate::services::object_store::find_location_dependents;
use crate::{Error, Result};
pub use unitycatalog_handlers::external_locations::ExternalLocationHandler;

#[async_trait::async_trait]
impl ExternalLocationHandler<RequestContext> for ServerHandler<RequestContext> {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_external_location(
        &self,
        request: CreateExternalLocationRequest,
        context: RequestContext,
    ) -> Result<ExternalLocation> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;

        // Local (file://) locations must sit within an allowed host root;
        // deny-by-default unless the server is configured otherwise. Cloud
        // schemes pass through.
        self.local_storage_policy()
            .check(&StorageLocationUrl::parse(&request.url)?)?;

        // The store resolves the credential and refuses a URL overlapping another
        // location: Unity Catalog forbids one external location from nesting
        // inside another (in either direction), so credential vending can resolve
        // a single owner for any storage path.
        let location = ExternalLocation {
            name: request.name,
            url: request.url,
            credential_name: request.credential_name,
            read_only: request.read_only.unwrap_or(false),
            comment: request.comment,
            ..Default::default()
        };

        // TODO: validate we can access the url with the provide credential

        self.storage_access()
            .create_external_location(location)
            .await
            .map_err(from_store)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_external_location(
        &self,
        request: DeleteExternalLocationRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        // Tables, volumes and agent skills under the location need it to vend
        // credentials, so it stays until they are gone unless `force` is set. The
        // check and the delete span two stores and are not atomic, like the
        // overlap check when those securables are created.
        if !request.force.unwrap_or(false) {
            let location = self
                .storage_access()
                .get_external_location(&request.name)
                .await
                .map_err(from_store)?;
            let dependents = find_location_dependents(self, &location.url, 5).await?;
            if !dependents.is_empty() {
                return Err(Error::invalid_argument(format!(
                    "external location '{}' is used by {}; delete them first or pass force",
                    request.name,
                    dependents
                        .iter()
                        .map(|name| format!("'{name}'"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
        }
        self.storage_access()
            .delete_external_location(&request.name)
            .await
            .map_err(from_store)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_external_location(
        &self,
        request: GetExternalLocationRequest,
        context: RequestContext,
    ) -> Result<ExternalLocation> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        self.storage_access()
            .get_external_location(&request.name)
            .await
            .map_err(from_store)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_external_locations(
        &self,
        request: ListExternalLocationsRequest,
        context: RequestContext,
    ) -> Result<ListExternalLocationsResponse> {
        self.check_required(&request, &context).await?;
        let page = Page::new(super::page_size(request.max_results)?, request.page_token);
        let (mut external_locations, next_page_token) = self
            .storage_access()
            .list_external_locations(page)
            .await
            .map_err(from_store)?;
        process_resources(self, &context, &Permission::Read, &mut external_locations).await?;
        Ok(ListExternalLocationsResponse {
            external_locations,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_external_location(
        &self,
        request: UpdateExternalLocationRequest,
        context: RequestContext,
    ) -> Result<ExternalLocation> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;

        // A new local (file://) URL must sit within an allowed root. The store
        // re-checks overlap (excluding this location) and resolves a new credential.
        if let Some(url) = &request.url {
            self.local_storage_policy()
                .check(&StorageLocationUrl::parse(url)?)?;
        }
        let update = ExternalLocationUpdate {
            new_name: request.new_name,
            url: request.url,
            credential_name: request.credential_name,
            read_only: request.read_only,
            comment: request.comment,
            owner: request.owner,
        };
        self.storage_access()
            .update_external_location(&request.name, update)
            .await
            .map_err(from_store)
    }
}

impl SecuredAction for CreateExternalLocationRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::external_location(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListExternalLocationsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::external_location(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetExternalLocationRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::external_location(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateExternalLocationRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::external_location(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteExternalLocationRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::external_location(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::credentials::v1::{
        AwsIamRoleConfig, CreateCredentialRequest, Purpose,
    };
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::api::CredentialHandler;
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

    /// A handler whose local-storage policy allows `root`. Only used by the
    /// POSIX-only allow-under-root test.
    #[cfg(not(windows))]
    fn handler_with_local_root(root: &std::path::Path) -> ServerHandler<RequestContext> {
        let local = crate::services::LocalStoragePolicy::new([root]).unwrap();
        handler().with_local_storage_policy(local)
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: crate::policy::Principal::anonymous(),
        }
    }

    async fn create_credential(h: &ServerHandler<RequestContext>, name: &str) {
        h.create_credential(
            CreateCredentialRequest {
                name: name.to_string(),
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
    }

    async fn create_location(
        h: &ServerHandler<RequestContext>,
        name: &str,
        url: &str,
    ) -> Result<ExternalLocation> {
        h.create_external_location(
            CreateExternalLocationRequest {
                name: name.to_string(),
                url: url.to_string(),
                credential_name: "cred".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
    }

    #[tokio::test]
    async fn file_location_denied_by_default() {
        // No local-storage policy configured ⇒ every file:// is rejected, even
        // though a cloud location at the same name would be accepted.
        let h = handler();
        create_credential(&h, "cred").await;
        let res = create_location(&h, "local", "file:///tmp/uc-denied").await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    // POSIX-only: allow-under-root matching depends on canonicalized paths that
    // don't line up on Windows (\\?\ prefix), and local file:// is gated off on
    // Windows anyway. Deny-by-default (above) is what matters there.
    #[cfg(not(windows))]
    #[tokio::test]
    async fn file_location_allowed_under_configured_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let h = handler_with_local_root(&root);
        create_credential(&h, "cred").await;

        // Under the allowed root: accepted.
        let inside = url::Url::from_directory_path(root.join("ext"))
            .unwrap()
            .to_string();
        create_location(&h, "ok", &inside).await.unwrap();

        // Outside the allowed root: rejected.
        let res = create_location(&h, "bad", "file:///etc/uc").await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");

        // Cloud schemes are unaffected by the local policy.
        create_location(&h, "cloud", "s3://bucket/data")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn rejects_overlapping_external_location() {
        let h = handler();
        create_credential(&h, "cred").await;

        // Base location succeeds.
        create_location(&h, "base", "s3://bucket/data")
            .await
            .unwrap();

        // A nested location (descendant) is rejected.
        let nested = create_location(&h, "nested", "s3://bucket/data/sub").await;
        assert!(
            matches!(nested, Err(Error::InvalidArgument(_))),
            "{nested:?}"
        );

        // An ancestor of the base location is rejected too (symmetric).
        let parent = create_location(&h, "parent", "s3://bucket").await;
        assert!(
            matches!(parent, Err(Error::InvalidArgument(_))),
            "{parent:?}"
        );

        // A sibling that merely shares a textual prefix is allowed.
        create_location(&h, "sibling", "s3://bucket/data-secret")
            .await
            .unwrap();

        // A disjoint location is allowed.
        create_location(&h, "other", "s3://bucket/other")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn update_to_overlapping_url_is_rejected_but_self_noop_ok() {
        let h = handler();
        create_credential(&h, "cred").await;
        create_location(&h, "base", "s3://bucket/data")
            .await
            .unwrap();
        create_location(&h, "other", "s3://bucket/other")
            .await
            .unwrap();

        // Updating `other` to overlap `base` is rejected.
        let res = h
            .update_external_location(
                UpdateExternalLocationRequest {
                    name: "other".to_string(),
                    url: Some("s3://bucket/data/inner".to_string()),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");

        // Re-saving `base` to its own URL is a no-op, not a self-overlap error.
        h.update_external_location(
            UpdateExternalLocationRequest {
                name: "base".to_string(),
                url: Some("s3://bucket/data".to_string()),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    async fn delete_location(
        h: &ServerHandler<RequestContext>,
        name: &str,
        force: Option<bool>,
    ) -> Result<()> {
        h.delete_external_location(
            DeleteExternalLocationRequest {
                name: name.to_string(),
                force,
                ..Default::default()
            },
            ctx(),
        )
        .await
    }

    async fn create_external_table(h: &ServerHandler<RequestContext>, name: &str, url: &str) {
        use crate::api::TableHandler;
        use unitycatalog_common::models::tables::v1::{
            CreateTableRequest, DataSourceFormat, TableType,
        };
        h.create_table(
            CreateTableRequest {
                name: name.to_string(),
                schema_name: "sch".to_string(),
                catalog_name: "cat".to_string(),
                table_type: TableType::External.into(),
                data_source_format: DataSourceFormat::Delta.into(),
                storage_location: Some(url.to_string()),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    /// A location governing a table can't be deleted until the table is gone,
    /// unless `force` is set; a table under a sibling path doesn't block it.
    #[tokio::test]
    async fn delete_refuses_location_in_use() {
        use crate::api::TableHandler;
        use unitycatalog_common::models::tables::v1::DeleteTableRequest;

        let h = handler();
        create_credential(&h, "cred").await;
        create_location(&h, "loc", "s3://bucket/loc").await.unwrap();
        create_location(&h, "loc2", "s3://bucket/loc2")
            .await
            .unwrap();
        create_external_table(&h, "t", "s3://bucket/loc/t").await;
        create_external_table(&h, "t2", "s3://bucket/loc2/t2").await;

        let res = delete_location(&h, "loc", None).await;
        assert!(
            matches!(&res, Err(Error::InvalidArgument(msg)) if msg.contains("'cat.sch.t'")),
            "{res:?}"
        );
        let res = delete_location(&h, "loc", Some(false)).await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");

        h.delete_table(
            DeleteTableRequest {
                full_name: "cat.sch.t".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        delete_location(&h, "loc", None)
            .await
            .expect("no dependents left");

        delete_location(&h, "loc2", Some(true))
            .await
            .expect("force bypasses the check");
    }
}
