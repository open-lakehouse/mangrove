//! Clients as handlers: serve a Unity Catalog resource from an upstream server.
//!
//! With the `handlers` feature every generated per-service client implements the
//! matching `olai-uc-handlers` trait (`CatalogServiceClient: CatalogHandler`, …),
//! forwarding each call to the server it points at. The impls are generated next
//! to each client (`codegen/<service>/handler.rs`); this module adds the
//! [`TemporaryCredentialHandler`] impl, which goes through the hand-written
//! [`TemporaryCredentialClient`], and the error conversion the impls rely on.
//!
//! The request context is ignored: the upstream call authenticates with the
//! client's own transport.

use unitycatalog_handlers::Error as HandlerError;

use crate::{Error, UcApiError};

/// Map a client error onto the handler error for the same condition, so an
/// upstream `404` surfaces as [`HandlerError::NotFound`] (and so on) rather than
/// as an opaque generic error.
impl From<Error> for HandlerError {
    fn from(err: Error) -> Self {
        match err {
            Error::Common { source } => HandlerError::Common { source },
            Error::Api(api) => from_api(api),
            Error::ClientError { source } => from_transport(source),
            Error::Delta(ref model) => {
                if err.is_not_found() {
                    HandlerError::NotFound
                } else if err.is_already_exists() {
                    HandlerError::AlreadyExists
                } else if err.is_permission_denied() {
                    HandlerError::NotAllowed
                } else if err.is_unauthenticated() {
                    HandlerError::Unauthenticated
                } else if err.is_commit_conflict() {
                    HandlerError::CommitVersionConflict(model.message.clone())
                } else if err.is_update_requirement_conflict() {
                    HandlerError::UpdateRequirementConflict(model.message.clone())
                } else if err.is_resource_exhausted() {
                    HandlerError::ResourceExhausted(model.message.clone())
                } else {
                    HandlerError::Generic(err.to_string())
                }
            }
            other => HandlerError::Generic(other.to_string()),
        }
    }
}

/// Servers report many conditions under resource-specific codes
/// (`CATALOG_DOES_NOT_EXIST`, `TABLE_ALREADY_EXISTS`, …) that parse to
/// [`UcApiError::Other`], so fall back to the HTTP status for those.
fn from_api(api: UcApiError) -> HandlerError {
    match api {
        UcApiError::InvalidParameter { message }
        | UcApiError::Other {
            status: 400,
            message,
            ..
        } => HandlerError::InvalidArgument(message),
        UcApiError::Unauthenticated { .. } | UcApiError::Other { status: 401, .. } => {
            HandlerError::Unauthenticated
        }
        UcApiError::PermissionDenied { .. } | UcApiError::Other { status: 403, .. } => {
            HandlerError::NotAllowed
        }
        UcApiError::NotFound { .. } | UcApiError::Other { status: 404, .. } => {
            HandlerError::NotFound
        }
        UcApiError::AlreadyExists { .. } | UcApiError::Other { status: 409, .. } => {
            HandlerError::AlreadyExists
        }
        UcApiError::RequestLimitExceeded { message }
        | UcApiError::Other {
            status: 429,
            message,
            ..
        } => HandlerError::ResourceExhausted(message),
        other => HandlerError::Generic(other.to_string()),
    }
}

/// The generated clients surface a non-2xx reply as an `olai_http` error that
/// wraps the retry layer's error, which still carries the status and body. Parse
/// the UC error envelope from those.
fn from_transport(err: olai_http::Error) -> HandlerError {
    use olai_http::Error as Http;
    let retry = match &err {
        Http::NotFound { source }
        | Http::AlreadyExists { source }
        | Http::Precondition { source }
        | Http::NotModified { source }
        | Http::PermissionDenied { source }
        | Http::Unauthenticated { source }
        | Http::Generic { source } => source.downcast_ref::<olai_http::RetryError>(),
        _ => None,
    };
    match retry.and_then(|retry| Some((retry.status()?, retry.body()))) {
        Some((status, body)) => HandlerError::from(crate::error::parse_error(
            status.as_u16(),
            body.map(str::as_bytes),
        )),
        None => HandlerError::Generic(err.to_string()),
    }
}

#[cfg(feature = "temporary_credentials")]
mod temporary_credentials {
    use async_trait::async_trait;
    use unitycatalog_common::models::temporary_credentials::v1::*;
    use unitycatalog_handlers::temporary_credentials::TemporaryCredentialHandler;

    use crate::TemporaryCredentialClient;

    /// Forwards through [`TemporaryCredentialClient`] rather than the generated
    /// low-level client, so responses that carry the inactive `credentials` oneof
    /// members as `null` still deserialize.
    #[async_trait]
    impl<Cx: Send + 'static> TemporaryCredentialHandler<Cx> for TemporaryCredentialClient {
        async fn generate_temporary_table_credentials(
            &self,
            request: GenerateTemporaryTableCredentialsRequest,
            _context: Cx,
        ) -> unitycatalog_handlers::Result<TemporaryCredential> {
            Ok(self
                .post_credential("temporary-table-credentials", &request)
                .await?)
        }

        async fn generate_temporary_path_credentials(
            &self,
            request: GenerateTemporaryPathCredentialsRequest,
            _context: Cx,
        ) -> unitycatalog_handlers::Result<TemporaryCredential> {
            Ok(self
                .post_credential("temporary-path-credentials", &request)
                .await?)
        }

        async fn generate_temporary_volume_credentials(
            &self,
            request: GenerateTemporaryVolumeCredentialsRequest,
            _context: Cx,
        ) -> unitycatalog_handlers::Result<TemporaryCredential> {
            Ok(self
                .post_credential("temporary-volume-credentials", &request)
                .await?)
        }

        async fn generate_temporary_model_version_credentials(
            &self,
            request: GenerateTemporaryModelVersionCredentialsRequest,
            _context: Cx,
        ) -> unitycatalog_handlers::Result<TemporaryCredential> {
            Ok(self
                .post_credential("temporary-model-version-credentials", &request)
                .await?)
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use unitycatalog_delta_api::models::{DeltaErrorModel, DeltaErrorType};

    use super::*;

    fn delta(error_type: DeltaErrorType) -> Error {
        Error::Delta(DeltaErrorModel {
            message: "upstream".to_string(),
            error_type,
            code: 409,
            stack: None,
        })
    }

    fn api(api: UcApiError) -> Error {
        Error::Api(api)
    }

    #[rstest]
    #[case(api(UcApiError::NotFound { message: "x".into() }), "NotFound")]
    #[case(api(UcApiError::AlreadyExists { message: "x".into() }), "AlreadyExists")]
    #[case(api(UcApiError::PermissionDenied { message: "x".into() }), "NotAllowed")]
    #[case(api(UcApiError::Unauthenticated { message: "x".into() }), "Unauthenticated")]
    #[case(api(UcApiError::InvalidParameter { message: "x".into() }), "InvalidArgument(\"x\")")]
    #[case(
        api(UcApiError::RequestLimitExceeded { message: "x".into() }),
        "ResourceExhausted(\"x\")"
    )]
    #[case(api(UcApiError::InternalError { message: "x".into() }), "Generic")]
    #[case(
        api(UcApiError::Other { status: 404, error_code: "TABLE_DOES_NOT_EXIST".into(), message: "x".into() }),
        "NotFound"
    )]
    #[case(delta(DeltaErrorType::NoSuchTableException), "NotFound")]
    #[case(
        delta(DeltaErrorType::CommitVersionConflictException),
        "CommitVersionConflict(\"upstream\")"
    )]
    #[case(Error::generic("boom"), "Generic(\"Generic error: boom\")")]
    fn test_client_error_maps_to_handler_error(#[case] err: Error, #[case] expected: &str) {
        let mapped = format!("{:?}", HandlerError::from(err));
        assert!(mapped.starts_with(expected), "{mapped} !~ {expected}");
    }
}

#[cfg(all(test, feature = "catalogs", feature = "temporary_credentials"))]
mod forwarding_tests {
    use mockito::Server;
    use unitycatalog_common::models::catalogs::v1::GetCatalogRequest;
    use unitycatalog_common::models::temporary_credentials::v1::GenerateTemporaryPathCredentialsRequest;
    use unitycatalog_handlers::catalogs::CatalogHandler;
    use unitycatalog_handlers::temporary_credentials::TemporaryCredentialHandler;
    use url::Url;

    use super::HandlerError;
    use crate::codegen::catalogs::CatalogServiceClient;
    use crate::{TemporaryCredentialClient, Transport};

    fn base_url(server: &Server) -> Url {
        Url::parse(&server.url()).unwrap()
    }

    #[tokio::test]
    async fn test_service_client_serves_as_handler() {
        let mut server = Server::new_async().await;
        let found = server
            .mock("GET", "/catalogs/main")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"main","comment":"upstream"}"#)
            .create_async()
            .await;
        let missing = server
            .mock("GET", "/catalogs/gone")
            .with_status(404)
            .with_body(r#"{"error_code":"CATALOG_DOES_NOT_EXIST","message":"no catalog"}"#)
            .create_async()
            .await;

        let client = CatalogServiceClient::new(Transport::new_unauthenticated(), base_url(&server));
        let handler: &dyn CatalogHandler<()> = &client;

        let request = |name: &str| GetCatalogRequest {
            name: name.to_string(),
            ..Default::default()
        };
        let catalog = handler.get_catalog(request("main"), ()).await.unwrap();
        assert_eq!(catalog.name, "main");
        assert_eq!(catalog.comment.as_deref(), Some("upstream"));

        let err = handler.get_catalog(request("gone"), ()).await.unwrap_err();
        assert!(matches!(err, HandlerError::NotFound), "{err:?}");

        found.assert_async().await;
        missing.assert_async().await;
    }

    #[tokio::test]
    async fn test_temporary_credential_handler_tolerates_null_oneof_members() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/temporary-path-credentials")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"expiration_time":1,"url":"s3://bucket/path",
                    "aws_temp_credentials":{"access_key_id":"ak","secret_access_key":"sk","session_token":"st"},
                    "azure_user_delegation_sas":null,"gcp_oauth_token":null}"#,
            )
            .create_async()
            .await;

        let client = TemporaryCredentialClient::new_with_url(
            Transport::new_unauthenticated(),
            base_url(&server),
        );
        let handler: &dyn TemporaryCredentialHandler<()> = &client;
        let credential = handler
            .generate_temporary_path_credentials(
                GenerateTemporaryPathCredentialsRequest {
                    url: "s3://bucket/path".to_string(),
                    ..Default::default()
                },
                (),
            )
            .await
            .unwrap();

        assert_eq!(credential.url, "s3://bucket/path");
        assert!(credential.credentials.is_some());
        mock.assert_async().await;
    }
}
