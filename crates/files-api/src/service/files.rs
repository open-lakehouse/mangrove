//! `FilesService` handlers, including streaming upload/download.

use bytes::Bytes;
use connectrpc::{
    ConnectError, RequestContext, Response, ServiceRequest, ServiceResult, ServiceStream,
    StreamMessage,
};
use futures::StreamExt;

use crate::error::StoreError;
use crate::service::AppState;
use crate::store::{ByteStream, ListOpts, Page};
use unitycatalog_files_proto::buffa::portal::files::v1::{
    CreateDirectoryRequest, DeleteDirectoryRequest, DeleteFileRequest, DirectoryEntry,
    DirectoryMetadata, DownloadFileRequest, DownloadFileResponse, FileMetadata,
    GetDirectoryMetadataRequest, GetFileMetadataRequest, ListDirectoryContentsRequest,
    ListDirectoryContentsResponse, ListDirectoryStreamRequest, UploadFileRequest,
    UploadFileResponse,
};
use unitycatalog_files_proto::connect::portal::files::v1::FilesService;

impl FilesService for AppState {
    async fn upload_file(
        &self,
        _ctx: RequestContext,
        mut requests: ServiceStream<StreamMessage<UploadFileRequest>>,
    ) -> ServiceResult<UploadFileResponse> {
        // The first message sets `path` (and optional `content_type`) and may
        // also carry the first chunk. Read it up front to resolve the
        // destination, then stream the first chunk + every later chunk straight
        // into the store's multipart upload — the file is never fully buffered.
        let first = requests
            .next()
            .await
            .ok_or_else(|| ConnectError::invalid_argument("upload stream was empty"))??
            .to_owned_message();
        if first.path.is_empty() {
            return Err(ConnectError::invalid_argument(
                "first message must set `path`",
            ));
        }
        let path = first.path;
        let content_type = first.content_type;

        // Stream of chunk bytes: the first message's chunk, then each subsequent
        // message's chunk. Later messages' `path`/`content_type` are ignored (set
        // only on the first, per the proto contract).
        let first_chunk =
            futures::stream::once(async move { Ok::<Bytes, StoreError>(Bytes::from(first.chunk)) });
        let rest = requests.map(|item| {
            item.map(|msg| Bytes::from(msg.to_owned_message().chunk))
                .map_err(|e| StoreError::Internal(format!("upload stream error: {e}")))
        });
        let byte_stream: ByteStream = first_chunk.chain(rest).boxed();

        let meta = self
            .files
            .put_file_stream(&path, content_type, byte_stream)
            .await?;
        Response::ok(UploadFileResponse {
            path: meta.path,
            file_size: meta.file_size,
            etag: meta.etag,
            ..Default::default()
        })
    }

    async fn download_file(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, DownloadFileRequest>,
    ) -> ServiceResult<ServiceStream<DownloadFileResponse>> {
        // Open a lazy byte stream from the store (object_store's own chunked
        // GET): bytes flow storage -> client without ever buffering the whole
        // file. Opening resolves not-found / bad-range up front; mid-transfer
        // errors surface as failed stream items, mapped to ConnectError.
        let byte_stream = self
            .files
            .read_file_stream(request.path, request.offset, request.length)
            .await?;

        let chunks = byte_stream.map(|res| {
            res.map(|chunk| DownloadFileResponse {
                chunk: chunk.to_vec(),
                ..Default::default()
            })
            .map_err(ConnectError::from)
        });

        Ok(Response::stream(chunks))
    }

    async fn delete_file(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, DeleteFileRequest>,
    ) -> ServiceResult<buffa_types::google::protobuf::Empty> {
        self.files.delete_file(request.path).await?;
        Response::ok(buffa_types::google::protobuf::Empty::default())
    }

    async fn get_file_metadata(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, GetFileMetadataRequest>,
    ) -> ServiceResult<FileMetadata> {
        Response::ok(self.files.stat_file(request.path).await?)
    }

    async fn create_directory(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, CreateDirectoryRequest>,
    ) -> ServiceResult<DirectoryMetadata> {
        Response::ok(self.files.create_directory(request.path).await?)
    }

    async fn delete_directory(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, DeleteDirectoryRequest>,
    ) -> ServiceResult<buffa_types::google::protobuf::Empty> {
        self.files.delete_directory(request.path).await?;
        Response::ok(buffa_types::google::protobuf::Empty::default())
    }

    async fn list_directory_contents(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, ListDirectoryContentsRequest>,
    ) -> ServiceResult<ListDirectoryContentsResponse> {
        let page = Page {
            max_results: request.max_results.map(|n| n.max(0) as usize),
            page_token: request.page_token.map(str::to_owned),
        };
        let (contents, next_page_token) = self.files.list_directory(request.path, page).await?;
        Response::ok(ListDirectoryContentsResponse {
            contents,
            next_page_token,
            ..Default::default()
        })
    }

    async fn get_directory_metadata(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, GetDirectoryMetadataRequest>,
    ) -> ServiceResult<DirectoryMetadata> {
        Response::ok(self.files.stat_directory(request.path).await?)
    }

    async fn list_directory_stream(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, ListDirectoryStreamRequest>,
    ) -> ServiceResult<ServiceStream<DirectoryEntry>> {
        // Open the store's lazy entry stream (object_store's streaming list for
        // the recursive case); entries flow to the client without the full
        // listing ever being materialized. Opening resolves the directory /
        // credential up front; mid-listing errors surface as failed stream items.
        let opts = ListOpts {
            recursive: request.recursive,
            start_after: request.start_after.map(str::to_owned),
            max_results: request.max_results.map(|n| n.max(0) as usize),
        };
        let entry_stream = self.files.list_files_opts(request.path, opts).await?;

        let entries = entry_stream.map(|res| res.map_err(ConnectError::from));
        Ok(Response::stream(entries))
    }
}
