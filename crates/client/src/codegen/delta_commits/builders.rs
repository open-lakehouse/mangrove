// @generated — do not edit by hand.
#![allow(unused_mut)]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
use super::client::*;
use crate::Result;
use std::future::IntoFuture;
use unitycatalog_common::models::delta_commits::v1::*;
/// Builder for commit
pub struct CommitBuilder {
    client: DeltaCommitClient,
    request: CommitRequest,
}
impl CommitBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `DeltaCommitClient`.
    pub(crate) fn new(
        client: DeltaCommitClient,
        table_id: impl Into<String>,
        table_uri: impl Into<String>,
    ) -> Self {
        let request = CommitRequest {
            table_id: table_id.into(),
            table_uri: table_uri.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// The commit to ratify. Absent for a backfill-only notification.
    pub fn with_commit_info(mut self, commit_info: impl Into<Option<CommitInfo>>) -> Self {
        self.request.commit_info = {
            let commit_info: ::core::option::Option<_> = commit_info.into();
            buffa::MessageField::from(commit_info)
        };
        self
    }
    /** Notify the catalog that commits up to and including this version have been
    published (backfilled) to the Delta log. The catalog prunes ratified
    commits accordingly.*/
    pub fn with_latest_backfilled_version(
        mut self,
        latest_backfilled_version: impl Into<Option<i64>>,
    ) -> Self {
        self.request.latest_backfilled_version = latest_backfilled_version.into();
        self
    }
    /// An optional Delta metadata change accompanying the commit.
    pub fn with_metadata(mut self, metadata: impl Into<Option<Metadata>>) -> Self {
        self.request.metadata = {
            let metadata: ::core::option::Option<_> = metadata.into();
            buffa::MessageField::from(metadata)
        };
        self
    }
}
impl IntoFuture for CommitBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.commit(&request).await })
    }
}
/// Builder for commits
pub struct GetCommitsBuilder {
    client: DeltaCommitClient,
    request: GetCommitsRequest,
}
impl GetCommitsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `DeltaCommitClient`.
    pub(crate) fn new(
        client: DeltaCommitClient,
        table_id: impl Into<String>,
        table_uri: impl Into<String>,
        start_version: i64,
    ) -> Self {
        let request = GetCommitsRequest {
            table_id: table_id.into(),
            table_uri: table_uri.into(),
            start_version,
            ..Default::default()
        };
        Self { client, request }
    }
    /** The highest version to return (inclusive). When set, must be
    `>= start_version`. Defaults to the latest version.*/
    pub fn with_end_version(mut self, end_version: impl Into<Option<i64>>) -> Self {
        self.request.end_version = end_version.into();
        self
    }
}
impl IntoFuture for GetCommitsBuilder {
    type Output = Result<GetCommitsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_commits(&request).await })
    }
}
