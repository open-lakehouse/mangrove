use itertools::Itertools;

use unitycatalog_common::models::external_locations::v1::*;
use unitycatalog_common::models::{
    ObjectLabel, ResourceExt, ResourceIdent, ResourceName, ResourceRef,
};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::external_locations::ExternalLocationHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::services::ProvidesLocalStoragePolicy;
use crate::services::location::StorageLocationUrl;
use crate::services::object_store::{list_external_locations, locations_overlap};
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext> + ProvidesLocalStoragePolicy>
    ExternalLocationHandler<RequestContext> for T
{
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

        // Reject locations that overlap an existing external location: Unity
        // Catalog forbids one external location from nesting inside another (in
        // either direction), so credential vending can resolve a single owner
        // for any storage path.
        check_no_overlap(self, &request.url, &request.name).await?;

        let mut resource = ExternalLocation {
            name: request.name,
            url: request.url,
            credential_name: request.credential_name,
            read_only: request.read_only.unwrap_or(false),
            comment: request.comment,
            ..Default::default()
        };
        let cred_ident = ResourceIdent::Credential(
            ResourceName::from_naive_str_split(&resource.credential_name).into(),
        );
        let (_credential, credential_ref) = self.get(&cred_ident).await?;
        if let ResourceRef::Uuid(uuid) = credential_ref {
            resource.credential_id = uuid.hyphenated().to_string();
        }

        // TODO: validate we can access the url with the provide credential

        let info = self.create(resource.into()).await?.0.try_into()?;
        Ok(info)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_external_location(
        &self,
        request: DeleteExternalLocationRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        // TODO: check if the location is used by any resources
        Ok(self.delete(&request.resource()).await?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_external_location(
        &self,
        request: GetExternalLocationRequest,
        context: RequestContext,
    ) -> Result<ExternalLocation> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;

        // TODO: populate relation fields (updated_* etc.)

        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_external_locations(
        &self,
        request: ListExternalLocationsRequest,
        context: RequestContext,
    ) -> Result<ListExternalLocationsResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::ExternalLocation,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListExternalLocationsResponse {
            external_locations: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
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

        let (current, _) = self.get(&request.resource()).await?;
        let curr_ident = current.resource_ident();
        let mut current: ExternalLocation = current.try_into()?;

        if let Some(name) = request.new_name {
            current.name = name;
        }
        if let Some(url) = request.url {
            // Re-validate overlap when the URL changes, excluding this
            // location's own record (matched by name) from the comparison.
            if url != current.url {
                // A new local (file://) URL must sit within an allowed root.
                self.local_storage_policy()
                    .check(&StorageLocationUrl::parse(&url)?)?;
                check_no_overlap(self, &url, &current.name).await?;
            }
            current.url = url;
        }
        if let Some(credential_name) = request.credential_name {
            current.credential_name = credential_name;
        }
        if let Some(read_only) = request.read_only {
            current.read_only = read_only;
        }
        if let Some(comment) = request.comment {
            current.comment = Some(comment);
        }

        // TODO:
        // - add update_* relations
        // - update owner if necessary

        Ok(self
            .update(&curr_ident, current.into())
            .await?
            .0
            .try_into()?)
    }
}

/// Reject `url` if it overlaps any existing external location other than the
/// one named `self_name` (so updating a location to its own URL is a no-op).
///
/// Unity Catalog forbids external locations from overlapping one another in
/// either direction (ancestor or descendant), matched on path-segment
/// boundaries.
async fn check_no_overlap(
    store: &(impl ResourceStore + ?Sized),
    url: &str,
    self_name: &str,
) -> Result<()> {
    let new_url = StorageLocationUrl::parse(url).map_err(|e| {
        Error::invalid_argument(format!("invalid external location url '{url}': {e}"))
    })?;
    for existing in list_external_locations(store).await? {
        if existing.name == self_name {
            continue;
        }
        let Ok(existing_url) = StorageLocationUrl::parse(&existing.url) else {
            continue;
        };
        if locations_overlap(&new_url, &existing_url) {
            return Err(Error::invalid_argument(format!(
                "external location url '{url}' overlaps existing external location '{}' ('{}')",
                existing.name, existing.url
            )));
        }
    }
    Ok(())
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
}
