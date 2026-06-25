// @generated — do not edit by hand.
#![allow(unused_mut)]
#![allow(unused_imports)]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
use super::client::*;
use crate::Result;
use std::future::IntoFuture;
use unitycatalog_common::models::temporary_credentials::v1::*;
/// Builder for temporary table credentials
pub struct GenerateTemporaryTableCredentialsBuilder {
    client: TemporaryCredentialClient,
    request: GenerateTemporaryTableCredentialsRequest,
}
impl GenerateTemporaryTableCredentialsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TemporaryCredentialClient`.
    pub(crate) fn new(
        client: TemporaryCredentialClient,
        table_id: impl Into<String>,
        operation: generate_temporary_table_credentials_request::Operation,
    ) -> Self {
        let request = GenerateTemporaryTableCredentialsRequest {
            table_id: table_id.into(),
            operation: operation as i32,
        };
        Self { client, request }
    }
}
impl IntoFuture for GenerateTemporaryTableCredentialsBuilder {
    type Output = Result<TemporaryCredential>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.generate_temporary_table_credentials(&request).await })
    }
}
/// Builder for temporary path credentials
pub struct GenerateTemporaryPathCredentialsBuilder {
    client: TemporaryCredentialClient,
    request: GenerateTemporaryPathCredentialsRequest,
}
impl GenerateTemporaryPathCredentialsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TemporaryCredentialClient`.
    pub(crate) fn new(
        client: TemporaryCredentialClient,
        url: impl Into<String>,
        operation: generate_temporary_path_credentials_request::Operation,
    ) -> Self {
        let request = GenerateTemporaryPathCredentialsRequest {
            url: url.into(),
            operation: operation as i32,
            ..Default::default()
        };
        Self { client, request }
    }
    /** When set to true, the service will not validate that the generated
    credentials can perform write operations, therefore no new paths will be
    created and the response will not contain valid credentials. Defaults to false.*/
    pub fn with_dry_run(mut self, dry_run: impl Into<Option<bool>>) -> Self {
        self.request.dry_run = dry_run.into();
        self
    }
}
impl IntoFuture for GenerateTemporaryPathCredentialsBuilder {
    type Output = Result<TemporaryCredential>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.generate_temporary_path_credentials(&request).await })
    }
}
/// Builder for temporary volume credentials
pub struct GenerateTemporaryVolumeCredentialsBuilder {
    client: TemporaryCredentialClient,
    request: GenerateTemporaryVolumeCredentialsRequest,
}
impl GenerateTemporaryVolumeCredentialsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TemporaryCredentialClient`.
    pub(crate) fn new(
        client: TemporaryCredentialClient,
        volume_id: impl Into<String>,
        operation: generate_temporary_volume_credentials_request::Operation,
    ) -> Self {
        let request = GenerateTemporaryVolumeCredentialsRequest {
            volume_id: volume_id.into(),
            operation: operation as i32,
        };
        Self { client, request }
    }
}
impl IntoFuture for GenerateTemporaryVolumeCredentialsBuilder {
    type Output = Result<TemporaryCredential>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.generate_temporary_volume_credentials(&request).await })
    }
}
