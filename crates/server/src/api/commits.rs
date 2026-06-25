//! Handler for the Delta catalog-managed commits ("commit coordinator") API.
//!
//! Implements the generated [`DeltaCommitHandler`] over any backend that exposes
//! a [`ResourceStore`], a [`Policy`], and a [`CommitCoordinator`]
//! ([`ProvidesCommitCoordinator`]). Request validation and error mapping mirror
//! the Unity Catalog OSS reference server; the arbitration state machine itself
//! lives in [`crate::services::commit_coordinator`].

use unitycatalog_common::models::delta_commits::v1::*;
use unitycatalog_common::models::tables::v1::{DataSourceFormat, Table, TableType};
use unitycatalog_common::models::{ResourceIdent, ResourceRef};

use super::RequestContext;
pub use crate::codegen::delta_commits::DeltaCommitHandler;
use crate::policy::{Permission, Policy};
use crate::services::location::StorageLocationUrl;
use crate::store::ResourceStore;
use crate::{Error, Result};
use unitycatalog_common::services::commit_coordinator::{CommitError, ProvidesCommitCoordinator};

/// Map a [`CommitError`] onto the server [`Error`] (and thus the HTTP status).
impl From<CommitError> for Error {
    fn from(err: CommitError) -> Self {
        match err {
            CommitError::VersionConflict(msg) => Error::CommitVersionConflict(msg),
            CommitError::InvalidArgument(msg) => Error::InvalidArgument(msg),
            CommitError::ResourceExhausted(msg) => Error::ResourceExhausted(msg),
            CommitError::Backend(msg) => Error::Generic(msg),
        }
    }
}

/// Resolve a catalog-managed table by UUID and validate it is eligible for
/// commit-coordination (MANAGED + DELTA + has a storage location).
///
/// Returns the resolved [`Table`]. Unknown tables surface as [`Error::NotFound`]
/// (404) from the store.
async fn resolve_managed_table<T>(handler: &T, table_id: &str) -> Result<Table>
where
    T: ResourceStore,
{
    let uuid = uuid::Uuid::parse_str(table_id)
        .map_err(|_| Error::invalid_argument("table_id is not a valid UUID"))?;
    let ident = ResourceIdent::Table(ResourceRef::Uuid(uuid));
    let (resource, _) = handler.get(&ident).await?;
    let table: Table = resource.try_into()?;

    if table.table_type != ::buffa::EnumValue::Known(TableType::Managed) {
        return Err(Error::invalid_argument(
            "only managed tables support catalog-managed commits",
        ));
    }
    if table.data_source_format != ::buffa::EnumValue::Known(DataSourceFormat::Delta) {
        return Err(Error::invalid_argument(
            "only delta tables support catalog-managed commits",
        ));
    }
    if table.storage_location.is_none() {
        return Err(Error::invalid_argument(
            "managed table does not have a storage location",
        ));
    }
    Ok(table)
}

/// Compare two storage URIs for equality after normalization.
fn uris_match(a: &str, b: &str) -> bool {
    match (StorageLocationUrl::parse(a), StorageLocationUrl::parse(b)) {
        (Ok(a), Ok(b)) => a.location() == b.location(),
        // Fall back to a raw comparison if either fails to parse as a known
        // object-store URL.
        _ => a == b,
    }
}

#[async_trait::async_trait]
impl<T> DeltaCommitHandler<RequestContext> for T
where
    T: ResourceStore + Policy<RequestContext> + ProvidesCommitCoordinator,
{
    #[tracing::instrument(skip(self, context))]
    async fn commit(&self, request: CommitRequest, context: RequestContext) -> Result<()> {
        let table_ident = ResourceIdent::Table(ResourceRef::Uuid(
            uuid::Uuid::parse_str(&request.table_id)
                .map_err(|_| Error::invalid_argument("table_id is not a valid UUID"))?,
        ));
        // Writing a commit requires write permission on the table.
        self.authorize_checked(&table_ident, &Permission::Write, &context)
            .await?;

        let table = resolve_managed_table(self, &request.table_id).await?;
        let storage_location = table
            .storage_location
            .as_deref()
            .expect("validated present in resolve_managed_table");
        if !uris_match(&request.table_uri, storage_location) {
            return Err(Error::invalid_argument(
                "table_uri does not match the table's storage location",
            ));
        }

        self.commit_coordinator()
            .commit(
                &request.table_id,
                request.commit_info.into_option(),
                request.latest_backfilled_version,
            )
            .await
            .map_err(Error::from)?;

        // Apply any metadata change carried by the commit to the stored table.
        // The reference's `updateTableFromCommit` persists the commit's metadata
        // (here: the configuration/property map) as the table's new desired state.
        if let Some(metadata) = request.metadata.into_option()
            && !metadata.configuration.is_empty()
        {
            let mut updated = table;
            updated.properties = metadata.configuration;
            self.update(&table_ident, updated.into()).await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, context))]
    async fn get_commits(
        &self,
        request: GetCommitsRequest,
        context: RequestContext,
    ) -> Result<GetCommitsResponse> {
        let table_ident = ResourceIdent::Table(ResourceRef::Uuid(
            uuid::Uuid::parse_str(&request.table_id)
                .map_err(|_| Error::invalid_argument("table_id is not a valid UUID"))?,
        ));
        self.authorize_checked(&table_ident, &Permission::Read, &context)
            .await?;

        // Validates MANAGED + DELTA + URL; unknown table -> 404. We deliberately
        // do not equality-check `table_uri` on the read path, matching UC OSS.
        resolve_managed_table(self, &request.table_id).await?;

        let (commits, latest_table_version) = self
            .commit_coordinator()
            .get_commits(
                &request.table_id,
                request.start_version,
                request.end_version,
            )
            .await
            .map_err(Error::from)?;

        Ok(GetCommitsResponse {
            commits,
            latest_table_version,
            ..Default::default()
        })
    }
}

#[cfg(all(test, feature = "memory"))]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::tables::v1::{DataSourceFormat, Table, TableType};
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::memory::InMemoryResourceStore;
    use crate::policy::{ConstantPolicy, Decision, Policy};
    use crate::services::ServerHandler;
    use crate::store::ResourceStore;

    const TABLE_URI: &str = "s3://bucket/warehouse/managed_table";

    async fn handler_with_table(
        policy: Decision,
        table_type: i32,
        format: i32,
        storage_location: Option<String>,
    ) -> (ServerHandler<RequestContext>, String) {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let table = Table {
            name: "managed_table".to_string(),
            catalog_name: "cat".to_string(),
            schema_name: "sch".to_string(),
            table_type: table_type.into(),
            data_source_format: format.into(),
            storage_location,
            ..Default::default()
        };
        let (_, table_ref) = store.create(table.into()).await.unwrap();
        let table_id = match table_ref {
            ResourceRef::Uuid(uuid) => uuid.to_string(),
            _ => panic!("expected uuid"),
        };
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::new(policy));
        let handler = ServerHandler::try_new_tokio(policy, store.clone(), store).unwrap();
        (handler, table_id)
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: crate::policy::Principal::anonymous(),
        }
    }

    fn commit_info(version: i64) -> CommitInfo {
        CommitInfo {
            version,
            timestamp: 1000 + version,
            file_name: format!("{version:020}.uuid.json"),
            file_size: 64,
            file_modification_timestamp: 2000 + version,
            ..Default::default()
        }
    }

    fn commit_request(table_id: &str, info: Option<CommitInfo>, lbv: Option<i64>) -> CommitRequest {
        CommitRequest {
            table_id: table_id.to_string(),
            table_uri: TABLE_URI.to_string(),
            commit_info: info.into(),
            latest_backfilled_version: lbv,
            metadata: ::buffa::MessageField::none(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn commit_then_get_commits_roundtrip() {
        let (handler, table_id) = handler_with_table(
            Decision::Allow,
            TableType::Managed as i32,
            DataSourceFormat::Delta as i32,
            Some(TABLE_URI.to_string()),
        )
        .await;

        handler
            .commit(commit_request(&table_id, Some(commit_info(1)), None), ctx())
            .await
            .unwrap();
        handler
            .commit(commit_request(&table_id, Some(commit_info(2)), None), ctx())
            .await
            .unwrap();

        let resp = handler
            .get_commits(
                GetCommitsRequest {
                    table_id: table_id.clone(),
                    table_uri: TABLE_URI.to_string(),
                    start_version: 0,
                    end_version: None,
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(resp.latest_table_version, 2);
        assert_eq!(
            resp.commits.iter().map(|c| c.version).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[tokio::test]
    async fn unknown_table_is_not_found() {
        let (handler, _) = handler_with_table(
            Decision::Allow,
            TableType::Managed as i32,
            DataSourceFormat::Delta as i32,
            Some(TABLE_URI.to_string()),
        )
        .await;
        let missing = uuid::Uuid::new_v4().to_string();
        let err = handler
            .commit(commit_request(&missing, Some(commit_info(1)), None), ctx())
            .await
            .unwrap_err();
        // The store reports the missing table as a not-found error (wrapped from
        // the common crate); both map to HTTP 404.
        assert!(
            matches!(
                err,
                Error::NotFound
                    | Error::Common {
                        source: unitycatalog_common::Error::NotFound
                    }
            ),
            "got {err:?}"
        );
    }

    #[tokio::test]
    async fn denied_policy_forbids_commit() {
        let (handler, table_id) = handler_with_table(
            Decision::Deny,
            TableType::Managed as i32,
            DataSourceFormat::Delta as i32,
            Some(TABLE_URI.to_string()),
        )
        .await;
        let err = handler
            .commit(commit_request(&table_id, Some(commit_info(1)), None), ctx())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotAllowed), "got {err:?}");
    }

    #[tokio::test]
    async fn mismatched_uri_is_invalid() {
        let (handler, table_id) = handler_with_table(
            Decision::Allow,
            TableType::Managed as i32,
            DataSourceFormat::Delta as i32,
            Some(TABLE_URI.to_string()),
        )
        .await;
        let mut req = commit_request(&table_id, Some(commit_info(1)), None);
        req.table_uri = "s3://bucket/warehouse/other_table".to_string();
        let err = handler.commit(req, ctx()).await.unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn non_managed_table_is_invalid() {
        let (handler, table_id) = handler_with_table(
            Decision::Allow,
            TableType::External as i32,
            DataSourceFormat::Delta as i32,
            Some(TABLE_URI.to_string()),
        )
        .await;
        let err = handler
            .commit(commit_request(&table_id, Some(commit_info(1)), None), ctx())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn version_conflict_maps_to_conflict_error() {
        let (handler, table_id) = handler_with_table(
            Decision::Allow,
            TableType::Managed as i32,
            DataSourceFormat::Delta as i32,
            Some(TABLE_URI.to_string()),
        )
        .await;
        handler
            .commit(commit_request(&table_id, Some(commit_info(1)), None), ctx())
            .await
            .unwrap();
        let err = handler
            .commit(commit_request(&table_id, Some(commit_info(1)), None), ctx())
            .await
            .unwrap_err();
        assert!(
            matches!(err, Error::CommitVersionConflict(_)),
            "got {err:?}"
        );
    }
}
