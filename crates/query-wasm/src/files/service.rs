//! In-wasm ConnectRPC service: the generated [`FilesService`] trait implemented
//! once over the volume [`engine`](super::engine) ops, plus the [`connect_unary`]
//! entry point the wasm-bindgen surface dispatches through.
//!
//! Mirrors hydrofoil's Tauri backend split exactly: the **unary metadata** RPCs
//! (`GetFileMetadata`, `ListDirectoryContents`, `DeleteFile`, `CreateDirectory`,
//! `DeleteDirectory`, `GetDirectoryMetadata`) run through the generic connect
//! dispatcher as **binary proto**; the **byte** RPCs (`UploadFile` /
//! `DownloadFile`) are `unimplemented!()` here and instead served by the native
//! wasm-bindgen exports in [`crate::bindings`] (a raw binary body, no proto
//! framing), because streaming file bytes through the connect envelope is the
//! path hydrofoil deliberately avoids.
//!
//! ## The `Send` shim
//!
//! `connectrpc`'s [`Handler`](connectrpc::Handler) bound requires the handler
//! future to be `Send` even on wasm32 (its `BoxFuture` alias is unconditionally
//! `+ Send`). The engine work is `!Send` — it drives the browser-Fetch-backed
//! `object_store`, whose futures are `!Send`. Each handler therefore wraps its
//! `!Send` body in [`SendWrapper`], which is sound on single-threaded wasm (the
//! future is only ever polled on the one thread), the same trick
//! [`engine::read_file`](super::engine::read_file) already uses for the store's
//! byte stream.

use std::sync::Arc;

use connectrpc::{
    CodecFormat, Payload, RequestContext, Router, ServiceRequest, ServiceResult,
    dispatcher::Dispatcher,
};
use http::HeaderMap;
use send_wrapper::SendWrapper;
use unitycatalog_object_store::UnityObjectStoreFactory;
use url::Url;

use crate::error::Error;
use crate::files::engine;
use crate::files::path::VolumePath;
use crate::generated::connect::portal::files::v1::{FilesService, FilesServiceExt};

use crate::generated::buffa::portal::files::v1 as pb;

/// The `FilesService` implementation: everything the dispatcher needs to build a
/// UC object-store factory on demand. Holds only the connection args (base URL +
/// optional bearer), so it is cheap to `Arc` and register per call.
pub struct FilesEngineService {
    base_url: Url,
    auth_token: Option<String>,
}

impl FilesEngineService {
    /// Build a service over the UC REST API at `base_url` with an optional bearer.
    pub fn new(base_url: Url, auth_token: Option<String>) -> Self {
        Self {
            base_url,
            auth_token,
        }
    }

    /// Build the canonical UC object-store factory once per op — the same
    /// construction the read path's [`crate::bindings`] `factory()` uses (browser
    /// Fetch transport; bearer via `with_token` when set, else the ambient
    /// session).
    async fn factory(&self) -> Result<UnityObjectStoreFactory, Error> {
        Ok(UnityObjectStoreFactory::builder()
            .with_uri(self.base_url.as_str())
            .with_token(self.auth_token.clone())
            .with_allow_unauthenticated(self.auth_token.is_none())
            .build()
            .await?)
    }
}

/// Map an engine [`Error`] to a `connectrpc` error, preserving the failure class
/// the read path already surfaces: a `conflict:`-tagged write becomes
/// `AlreadyExists`, a `network/CORS:`-tagged transport failure becomes
/// `Unavailable`, everything else `Internal`. The JS edge re-derives its
/// `code` contract from the connect error code.
fn to_connect_error(err: Error) -> connectrpc::ConnectError {
    let message = err.to_string();
    let lower = message.to_ascii_lowercase();
    if lower.contains("conflict:") {
        connectrpc::ConnectError::already_exists(message)
    } else if lower.contains("network/cors") {
        connectrpc::ConnectError::unavailable(message)
    } else {
        connectrpc::ConnectError::internal(message)
    }
}

// The generated trait declares each method as `-> impl Future<Output = ...> +
// Send`; implementing with `async fn` (as hydrofoil does) desugars to a more
// refined return type, which fires the `refining_impl_trait` lint. The refinement
// is intentional and sound — callers only see the trait's erased type — so allow
// it rather than hand-spell every `impl Future` bound.
#[allow(refining_impl_trait)]
impl FilesService for FilesEngineService {
    async fn upload_file(
        &self,
        _ctx: RequestContext,
        _requests: connectrpc::ServiceStream<connectrpc::StreamMessage<pb::UploadFileRequest>>,
    ) -> ServiceResult<pb::UploadFileResponse> {
        // Bytes bypass the dispatcher: `writeFileBytes` in `bindings` serves
        // uploads natively (a raw binary body), matching hydrofoil.
        Err(connectrpc::ConnectError::unimplemented(
            "UploadFile is served by the native writeFileBytes export, not the connect dispatcher",
        ))
    }

    async fn download_file(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, pb::DownloadFileRequest>,
    ) -> ServiceResult<connectrpc::ServiceStream<pb::DownloadFileResponse>> {
        Err(connectrpc::ConnectError::unimplemented(
            "DownloadFile is served by the native readFileBytes export, not the connect dispatcher",
        ))
    }

    async fn delete_file(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, pb::DeleteFileRequest>,
    ) -> ServiceResult<::buffa_types::google::protobuf::Empty> {
        let path = request.path.to_string();
        SendWrapper::new(async move {
            let parsed = VolumePath::parse(&path).map_err(to_connect_error)?;
            let factory = self.factory().await.map_err(to_connect_error)?;
            engine::delete_file(&factory, &parsed)
                .await
                .map_err(to_connect_error)?;
            connectrpc::Response::ok(::buffa_types::google::protobuf::Empty::default())
        })
        .await
    }

    async fn get_file_metadata(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, pb::GetFileMetadataRequest>,
    ) -> ServiceResult<pb::FileMetadata> {
        let path = request.path.to_string();
        SendWrapper::new(async move {
            let parsed = VolumePath::parse(&path).map_err(to_connect_error)?;
            let factory = self.factory().await.map_err(to_connect_error)?;
            let meta = engine::stat(&factory, &parsed)
                .await
                .map_err(to_connect_error)?;
            connectrpc::Response::ok(meta)
        })
        .await
    }

    async fn create_directory(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, pb::CreateDirectoryRequest>,
    ) -> ServiceResult<pb::DirectoryMetadata> {
        let path = request.path.to_string();
        SendWrapper::new(async move {
            let parsed = VolumePath::parse(&path).map_err(to_connect_error)?;
            let factory = self.factory().await.map_err(to_connect_error)?;
            let meta = engine::create_dir(&factory, &parsed)
                .await
                .map_err(to_connect_error)?;
            connectrpc::Response::ok(meta)
        })
        .await
    }

    async fn delete_directory(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, pb::DeleteDirectoryRequest>,
    ) -> ServiceResult<::buffa_types::google::protobuf::Empty> {
        let path = request.path.to_string();
        SendWrapper::new(async move {
            let parsed = VolumePath::parse(&path).map_err(to_connect_error)?;
            let factory = self.factory().await.map_err(to_connect_error)?;
            engine::delete_dir(&factory, &parsed)
                .await
                .map_err(to_connect_error)?;
            connectrpc::Response::ok(::buffa_types::google::protobuf::Empty::default())
        })
        .await
    }

    async fn list_directory_contents(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, pb::ListDirectoryContentsRequest>,
    ) -> ServiceResult<pb::ListDirectoryContentsResponse> {
        let path = request.path.to_string();
        // `max_results` is a proto `int32`; the engine wants a `u32` page size.
        // Clamp a negative to `None` (treated as "no cap") rather than error.
        let max_results = request.max_results.and_then(|n| u32::try_from(n).ok());
        let page_token = request.page_token.map(str::to_string);
        SendWrapper::new(async move {
            let parsed = VolumePath::parse(&path).map_err(to_connect_error)?;
            let factory = self.factory().await.map_err(to_connect_error)?;
            let page = engine::list_directory(&factory, &parsed, max_results, page_token)
                .await
                .map_err(to_connect_error)?;
            let contents = page
                .entries
                .into_iter()
                .map(|e| pb::DirectoryEntry {
                    path: e.path,
                    is_directory: e.is_directory,
                    file_size: e.file_size as i64,
                    last_modified: e.last_modified,
                    ..Default::default()
                })
                .collect();
            connectrpc::Response::ok(pb::ListDirectoryContentsResponse {
                contents,
                next_page_token: page.next_page_token,
                ..Default::default()
            })
        })
        .await
    }

    async fn list_directory_stream(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, pb::ListDirectoryStreamRequest>,
    ) -> ServiceResult<connectrpc::ServiceStream<pb::DirectoryEntry>> {
        // The UI drives the paged unary `ListDirectoryContents`; the streaming
        // variant is not wired on wasm (no server-streaming feature in the
        // browser build).
        Err(connectrpc::ConnectError::unimplemented(
            "ListDirectoryStream is not served on wasm; use ListDirectoryContents",
        ))
    }

    async fn get_directory_metadata(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, pb::GetDirectoryMetadataRequest>,
    ) -> ServiceResult<pb::DirectoryMetadata> {
        let path = request.path.to_string();
        // A directory has no object to HEAD; report the canonical path back with
        // an unknown mtime (`0`), matching `create_dir`'s advisory metadata.
        SendWrapper::new(async move {
            let parsed = VolumePath::parse(&path).map_err(to_connect_error)?;
            connectrpc::Response::ok(pb::DirectoryMetadata {
                path: parsed.to_canonical(),
                last_modified: 0,
                ..Default::default()
            })
        })
        .await
    }
}

/// Dispatch one unary ConnectRPC call through the [`FilesEngineService`] router.
///
/// `path` is the full RPC path (`portal.files.v1.FilesService/GetFileMetadata`);
/// `request_bytes` is the binary-proto request body; the returned `Bytes` are the
/// binary-proto response body. This is the single seam the wasm-bindgen
/// `connectUnary` export drives — it mirrors hydrofoil's `connect_unary_proto`
/// Tauri command.
///
/// Only the metadata RPCs resolve here; the byte RPCs (`UploadFile` /
/// `DownloadFile`) return `Unimplemented` (they are served natively).
pub async fn connect_unary(
    base_url: Url,
    auth_token: Option<String>,
    path: &str,
    request_bytes: bytes::Bytes,
) -> Result<bytes::Bytes, connectrpc::ConnectError> {
    let service = Arc::new(FilesEngineService::new(base_url, auth_token));
    let router = service.register(Router::new());
    let ctx = RequestContext::new(HeaderMap::new()).with_path(path);
    let payload = Payload::new(request_bytes, CodecFormat::Proto);
    let response = router
        .call_unary(path, ctx, payload, CodecFormat::Proto)
        .await?;
    Ok(response.body)
}
