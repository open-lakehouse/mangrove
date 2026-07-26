///Shorthand for `OwnedView<UploadFileRequestView<'static>>`.
pub type OwnedUploadFileRequestView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::UploadFileRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<UploadFileResponseView<'static>>`.
pub type OwnedUploadFileResponseView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::UploadFileResponseView<
        'static,
    >,
>;
///Shorthand for `OwnedView<DownloadFileRequestView<'static>>`.
pub type OwnedDownloadFileRequestView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::DownloadFileRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<DownloadFileResponseView<'static>>`.
pub type OwnedDownloadFileResponseView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::DownloadFileResponseView<
        'static,
    >,
>;
///Shorthand for `OwnedView<DeleteFileRequestView<'static>>`.
pub type OwnedDeleteFileRequestView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::DeleteFileRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<EmptyView<'static>>`.
pub type OwnedEmptyView = ::buffa::view::OwnedView<
    ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
>;
///Shorthand for `OwnedView<GetFileMetadataRequestView<'static>>`.
pub type OwnedGetFileMetadataRequestView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::GetFileMetadataRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<FileMetadataView<'static>>`.
pub type OwnedFileMetadataView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::FileMetadataView<'static>,
>;
///Shorthand for `OwnedView<CreateDirectoryRequestView<'static>>`.
pub type OwnedCreateDirectoryRequestView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::CreateDirectoryRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<DirectoryMetadataView<'static>>`.
pub type OwnedDirectoryMetadataView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryMetadataView<
        'static,
    >,
>;
///Shorthand for `OwnedView<DeleteDirectoryRequestView<'static>>`.
pub type OwnedDeleteDirectoryRequestView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::DeleteDirectoryRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<ListDirectoryContentsRequestView<'static>>`.
pub type OwnedListDirectoryContentsRequestView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryContentsRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<ListDirectoryContentsResponseView<'static>>`.
pub type OwnedListDirectoryContentsResponseView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryContentsResponseView<
        'static,
    >,
>;
///Shorthand for `OwnedView<ListDirectoryStreamRequestView<'static>>`.
pub type OwnedListDirectoryStreamRequestView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryStreamRequestView<
        'static,
    >,
>;
///Shorthand for `OwnedView<DirectoryEntryView<'static>>`.
pub type OwnedDirectoryEntryView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryEntryView<
        'static,
    >,
>;
///Shorthand for `OwnedView<GetDirectoryMetadataRequestView<'static>>`.
pub type OwnedGetDirectoryMetadataRequestView = ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::GetDirectoryMetadataRequestView<
        'static,
    >,
>;
impl ::connectrpc::Encodable<
    crate::generated::buffa::portal::files::v1::UploadFileResponse,
>
for crate::generated::buffa::portal::files::v1::__buffa::view::UploadFileResponseView<
    '_,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<
    crate::generated::buffa::portal::files::v1::UploadFileResponse,
>
for ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::UploadFileResponseView<
        'static,
    >,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<
    crate::generated::buffa::portal::files::v1::DownloadFileResponse,
>
for crate::generated::buffa::portal::files::v1::__buffa::view::DownloadFileResponseView<
    '_,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<
    crate::generated::buffa::portal::files::v1::DownloadFileResponse,
>
for ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::DownloadFileResponseView<
        'static,
    >,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::generated::buffa::portal::files::v1::FileMetadata>
for crate::generated::buffa::portal::files::v1::__buffa::view::FileMetadataView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::generated::buffa::portal::files::v1::FileMetadata>
for ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::FileMetadataView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<
    crate::generated::buffa::portal::files::v1::DirectoryMetadata,
>
for crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryMetadataView<
    '_,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<
    crate::generated::buffa::portal::files::v1::DirectoryMetadata,
>
for ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryMetadataView<
        'static,
    >,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<
    crate::generated::buffa::portal::files::v1::ListDirectoryContentsResponse,
>
for crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryContentsResponseView<
    '_,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<
    crate::generated::buffa::portal::files::v1::ListDirectoryContentsResponse,
>
for ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryContentsResponseView<
        'static,
    >,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::generated::buffa::portal::files::v1::DirectoryEntry>
for crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryEntryView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::generated::buffa::portal::files::v1::DirectoryEntry>
for ::buffa::view::OwnedView<
    crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryEntryView<
        'static,
    >,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
/// Full service name for this service.
pub const FILES_SERVICE_SERVICE_NAME: &str = "portal.files.v1.FilesService";
/// Static [`Spec`](::connectrpc::Spec) for the server-side `UploadFile` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const FILES_SERVICE_UPLOAD_FILE_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/portal.files.v1.FilesService/UploadFile",
        ::connectrpc::StreamType::ClientStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `DownloadFile` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const FILES_SERVICE_DOWNLOAD_FILE_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/portal.files.v1.FilesService/DownloadFile",
        ::connectrpc::StreamType::ServerStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `DeleteFile` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const FILES_SERVICE_DELETE_FILE_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/portal.files.v1.FilesService/DeleteFile",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `GetFileMetadata` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const FILES_SERVICE_GET_FILE_METADATA_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/portal.files.v1.FilesService/GetFileMetadata",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `CreateDirectory` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const FILES_SERVICE_CREATE_DIRECTORY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/portal.files.v1.FilesService/CreateDirectory",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `DeleteDirectory` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const FILES_SERVICE_DELETE_DIRECTORY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/portal.files.v1.FilesService/DeleteDirectory",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `ListDirectoryContents` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const FILES_SERVICE_LIST_DIRECTORY_CONTENTS_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/portal.files.v1.FilesService/ListDirectoryContents",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `ListDirectoryStream` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const FILES_SERVICE_LIST_DIRECTORY_STREAM_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/portal.files.v1.FilesService/ListDirectoryStream",
        ::connectrpc::StreamType::ServerStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `GetDirectoryMetadata` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const FILES_SERVICE_GET_DIRECTORY_METADATA_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/portal.files.v1.FilesService/GetDirectoryMetadata",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// File and directory operations.
/// Binary transfer uses streaming RPCs: UploadFile is a client stream (chunks in)
/// and DownloadFile is a server stream (chunks out). Metadata and directory
/// operations are unary.
///
/// # Implementing handlers
///
/// Implement methods with plain `async fn`; the returned future satisfies
/// the `Send` bound automatically.
///
/// **Unary and server-streaming requests** arrive as
/// [`ServiceRequest<'_, Req>`](::connectrpc::ServiceRequest): a zero-copy
/// view of the request plus its body, valid for the duration of the call.
/// Fields are read directly (`request.name` is a `&str` into the decoded
/// buffer) and the borrow may be held across `.await` points. Anything
/// that must outlive the call — `tokio::spawn`, channels, server state,
/// or data captured by a returned response stream — takes owned data:
/// call `request.to_owned_message()` (or copy the specific fields)
/// first.
///
/// **Client-streaming and bidi requests** arrive as
/// `ServiceStream<`[`StreamMessage<Req>`](::connectrpc::StreamMessage)`>`.
/// Each item owns its decoded buffer and is `Send + 'static`, so items
/// can be buffered or moved into spawned tasks; read fields zero-copy
/// through the generated accessor methods (`item.name()`) or `.view()`,
/// convert with `.to_owned_message()`, or yield an item back unchanged —
/// `StreamMessage<M>` implements `Encodable<M>`.
///
/// Request types resolved through `extern_path` (e.g. well-known types
/// from another crate) use the same wrappers; the crate that owns the
/// type must be generated with buffa ≥ 0.7.0 and views enabled so the
/// backing `HasMessageView` impl exists.
///
/// The `impl Encodable<Out>` return bound accepts the owned `Out`, the
/// generated `OutView<'_>` / `OwnedOutView`,
/// [`MaybeBorrowed`](::connectrpc::MaybeBorrowed), or
/// [`PreEncoded`](::connectrpc::PreEncoded) for handlers that encode a
/// non-`'static` view internally and pass the bytes across the handler
/// boundary. View bodies are not emitted for output types mapped via
/// `extern_path` (the impl would be an orphan); return owned for
/// WKT/extern outputs.
///
/// Server-streaming and bidi-streaming methods return
/// `ServiceStream<impl Encodable<Out> + Send + use<Self>>`. The
/// `use<Self>` precise-capturing clause excludes `&self`'s lifetime and
/// the request's lifetime (unary methods use `use<'a, Self>` and may
/// borrow from `&self`), so stream items must be `'static` and cannot
/// borrow from the request. To stream view-encoded data, encode each
/// item inside the stream body and yield
/// [`PreEncoded`](::connectrpc::PreEncoded) — see its `# Streaming
/// example` doc.
#[allow(clippy::type_complexity)]
pub trait FilesService: Send + Sync + 'static {
    /// Upload a file. Client-streaming: send the path on the first message, then
    /// stream the contents as chunks.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// Each `requests` item is a [`StreamMessage`](::connectrpc::StreamMessage):
    /// it owns its buffer, is `Send + 'static`, and exposes zero-copy
    /// accessor methods (`item.name()`), `.view()`, and
    /// `.to_owned_message()`.
    fn upload_file<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::ServiceStream<
            ::connectrpc::StreamMessage<
                crate::generated::buffa::portal::files::v1::UploadFileRequest,
            >,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::generated::buffa::portal::files::v1::UploadFileResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Download a file. Server-streaming: the contents arrive as ordered chunks.
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call (until the response stream is returned);
    /// message fields are read directly on it (zero-copy). Data the
    /// returned stream needs must be copied out or converted via
    /// `.to_owned_message()`.
    fn download_file(
        &self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::generated::buffa::portal::files::v1::DownloadFileRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::generated::buffa::portal::files::v1::DownloadFileResponse,
                > + Send + use<Self>,
            >,
        >,
    > + Send;
    /// Delete a file.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn delete_file<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::generated::buffa::portal::files::v1::DeleteFileRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                ::buffa_types::google::protobuf::Empty,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Get file metadata (the proto analog of an HTTP HEAD on a file).
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn get_file_metadata<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::generated::buffa::portal::files::v1::GetFileMetadataRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::generated::buffa::portal::files::v1::FileMetadata,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Create a directory.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn create_directory<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::generated::buffa::portal::files::v1::CreateDirectoryRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::generated::buffa::portal::files::v1::DirectoryMetadata,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Delete a directory.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn delete_directory<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::generated::buffa::portal::files::v1::DeleteDirectoryRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                ::buffa_types::google::protobuf::Empty,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// List the contents of a directory. Unary + paged: returns one bounded page,
    /// suitable for a UI. For a large directory, prefer StreamDirectory.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn list_directory_contents<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::generated::buffa::portal::files::v1::ListDirectoryContentsRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::generated::buffa::portal::files::v1::ListDirectoryContentsResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Stream the contents of a directory. Server-streaming: entries arrive lazily
    /// so a directory with many files is never fully materialized.
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call (until the response stream is returned);
    /// message fields are read directly on it (zero-copy). Data the
    /// returned stream needs must be copied out or converted via
    /// `.to_owned_message()`.
    fn list_directory_stream(
        &self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::generated::buffa::portal::files::v1::ListDirectoryStreamRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::generated::buffa::portal::files::v1::DirectoryEntry,
                > + Send + use<Self>,
            >,
        >,
    > + Send;
    /// Get directory metadata (the proto analog of an HTTP HEAD on a directory).
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn get_directory_metadata<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::generated::buffa::portal::files::v1::GetDirectoryMetadataRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::generated::buffa::portal::files::v1::DirectoryMetadata,
            > + Send + use<'a, Self>,
        >,
    > + Send;
}
/// Extension trait for registering a service implementation with a Router.
///
/// This trait is automatically implemented for all types that implement the service trait.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// let service = Arc::new(MyServiceImpl);
/// let router = service.register(Router::new());
/// ```
pub trait FilesServiceExt: FilesService {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: FilesService> FilesServiceExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view_client_stream(
                FILES_SERVICE_SERVICE_NAME,
                "UploadFile",
                ::connectrpc::view_client_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |ctx, req, format| {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let req = ::connectrpc::dispatcher::codegen::into_stream_messages::<
                                crate::generated::buffa::portal::files::v1::UploadFileRequest,
                            >(req);
                            svc.upload_file(ctx, req)
                                .await?
                                .encode::<
                                    crate::generated::buffa::portal::files::v1::UploadFileResponse,
                                >(format)
                        }
                    }
                }),
            )
            .with_spec(FILES_SERVICE_UPLOAD_FILE_SPEC)
            .route_view_server_stream::<
                _,
                _,
                crate::generated::buffa::portal::files::v1::DownloadFileResponse,
            >(
                FILES_SERVICE_SERVICE_NAME,
                "DownloadFile",
                ::connectrpc::view_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::generated::buffa::portal::files::v1::__buffa::view::DownloadFileRequestView<
                                'static,
                            >,
                        >|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::generated::buffa::portal::files::v1::DownloadFileRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.download_file(ctx, sreq).await
                        }
                    }
                }),
            )
            .with_spec(FILES_SERVICE_DOWNLOAD_FILE_SPEC)
            .route_view(
                FILES_SERVICE_SERVICE_NAME,
                "DeleteFile",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::generated::buffa::portal::files::v1::__buffa::view::DeleteFileRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::generated::buffa::portal::files::v1::DeleteFileRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.delete_file(ctx, sreq)
                                .await?
                                .encode::<::buffa_types::google::protobuf::Empty>(format)
                        }
                    })
                },
            )
            .with_spec(FILES_SERVICE_DELETE_FILE_SPEC)
            .route_view(
                FILES_SERVICE_SERVICE_NAME,
                "GetFileMetadata",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::generated::buffa::portal::files::v1::__buffa::view::GetFileMetadataRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::generated::buffa::portal::files::v1::GetFileMetadataRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.get_file_metadata(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::generated::buffa::portal::files::v1::FileMetadata,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(FILES_SERVICE_GET_FILE_METADATA_SPEC)
            .route_view(
                FILES_SERVICE_SERVICE_NAME,
                "CreateDirectory",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::generated::buffa::portal::files::v1::__buffa::view::CreateDirectoryRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::generated::buffa::portal::files::v1::CreateDirectoryRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.create_directory(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::generated::buffa::portal::files::v1::DirectoryMetadata,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(FILES_SERVICE_CREATE_DIRECTORY_SPEC)
            .route_view(
                FILES_SERVICE_SERVICE_NAME,
                "DeleteDirectory",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::generated::buffa::portal::files::v1::__buffa::view::DeleteDirectoryRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::generated::buffa::portal::files::v1::DeleteDirectoryRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.delete_directory(ctx, sreq)
                                .await?
                                .encode::<::buffa_types::google::protobuf::Empty>(format)
                        }
                    })
                },
            )
            .with_spec(FILES_SERVICE_DELETE_DIRECTORY_SPEC)
            .route_view(
                FILES_SERVICE_SERVICE_NAME,
                "ListDirectoryContents",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryContentsRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::generated::buffa::portal::files::v1::ListDirectoryContentsRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.list_directory_contents(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::generated::buffa::portal::files::v1::ListDirectoryContentsResponse,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(FILES_SERVICE_LIST_DIRECTORY_CONTENTS_SPEC)
            .route_view_server_stream::<
                _,
                _,
                crate::generated::buffa::portal::files::v1::DirectoryEntry,
            >(
                FILES_SERVICE_SERVICE_NAME,
                "ListDirectoryStream",
                ::connectrpc::view_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryStreamRequestView<
                                'static,
                            >,
                        >|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::generated::buffa::portal::files::v1::ListDirectoryStreamRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.list_directory_stream(ctx, sreq).await
                        }
                    }
                }),
            )
            .with_spec(FILES_SERVICE_LIST_DIRECTORY_STREAM_SPEC)
            .route_view(
                FILES_SERVICE_SERVICE_NAME,
                "GetDirectoryMetadata",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::generated::buffa::portal::files::v1::__buffa::view::GetDirectoryMetadataRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::generated::buffa::portal::files::v1::GetDirectoryMetadataRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.get_directory_metadata(ctx, sreq)
                                .await?
                                .encode::<
                                    crate::generated::buffa::portal::files::v1::DirectoryMetadata,
                                >(format)
                        }
                    })
                },
            )
            .with_spec(FILES_SERVICE_GET_DIRECTORY_METADATA_SPEC)
    }
}
/// Monomorphic dispatcher for `FilesService`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = FilesServiceServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct FilesServiceServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: FilesService> FilesServiceServer<T> {
    /// Wrap a service implementation in a monomorphic dispatcher.
    pub fn new(service: T) -> Self {
        Self {
            inner: ::std::sync::Arc::new(service),
        }
    }
    /// Wrap an already-`Arc`'d service implementation.
    pub fn from_arc(inner: ::std::sync::Arc<T>) -> Self {
        Self { inner }
    }
}
impl<T> Clone for FilesServiceServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: FilesService> ::connectrpc::Dispatcher for FilesServiceServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("portal.files.v1.FilesService/")?;
        match method {
            "UploadFile" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::client_streaming()
                        .with_spec(FILES_SERVICE_UPLOAD_FILE_SPEC),
                )
            }
            "DownloadFile" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::server_streaming()
                        .with_spec(FILES_SERVICE_DOWNLOAD_FILE_SPEC),
                )
            }
            "DeleteFile" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(FILES_SERVICE_DELETE_FILE_SPEC),
                )
            }
            "GetFileMetadata" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(FILES_SERVICE_GET_FILE_METADATA_SPEC),
                )
            }
            "CreateDirectory" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(FILES_SERVICE_CREATE_DIRECTORY_SPEC),
                )
            }
            "DeleteDirectory" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(FILES_SERVICE_DELETE_DIRECTORY_SPEC),
                )
            }
            "ListDirectoryContents" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(FILES_SERVICE_LIST_DIRECTORY_CONTENTS_SPEC),
                )
            }
            "ListDirectoryStream" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::server_streaming()
                        .with_spec(FILES_SERVICE_LIST_DIRECTORY_STREAM_SPEC),
                )
            }
            "GetDirectoryMetadata" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(FILES_SERVICE_GET_DIRECTORY_METADATA_SPEC),
                )
            }
            _ => None,
        }
    }
    fn call_unary(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::Payload,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("portal.files.v1.FilesService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "DeleteFile" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::generated::buffa::portal::files::v1::DeleteFileRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::generated::buffa::portal::files::v1::__buffa::view::DeleteFileRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::generated::buffa::portal::files::v1::DeleteFileRequest,
                    >::from_parts(&req, &body);
                    svc.delete_file(ctx, req)
                        .await?
                        .encode::<::buffa_types::google::protobuf::Empty>(format)
                })
            }
            "GetFileMetadata" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::generated::buffa::portal::files::v1::GetFileMetadataRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::generated::buffa::portal::files::v1::__buffa::view::GetFileMetadataRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::generated::buffa::portal::files::v1::GetFileMetadataRequest,
                    >::from_parts(&req, &body);
                    svc.get_file_metadata(ctx, req)
                        .await?
                        .encode::<
                            crate::generated::buffa::portal::files::v1::FileMetadata,
                        >(format)
                })
            }
            "CreateDirectory" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::generated::buffa::portal::files::v1::CreateDirectoryRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::generated::buffa::portal::files::v1::__buffa::view::CreateDirectoryRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::generated::buffa::portal::files::v1::CreateDirectoryRequest,
                    >::from_parts(&req, &body);
                    svc.create_directory(ctx, req)
                        .await?
                        .encode::<
                            crate::generated::buffa::portal::files::v1::DirectoryMetadata,
                        >(format)
                })
            }
            "DeleteDirectory" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::generated::buffa::portal::files::v1::DeleteDirectoryRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::generated::buffa::portal::files::v1::__buffa::view::DeleteDirectoryRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::generated::buffa::portal::files::v1::DeleteDirectoryRequest,
                    >::from_parts(&req, &body);
                    svc.delete_directory(ctx, req)
                        .await?
                        .encode::<::buffa_types::google::protobuf::Empty>(format)
                })
            }
            "ListDirectoryContents" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::generated::buffa::portal::files::v1::ListDirectoryContentsRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryContentsRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::generated::buffa::portal::files::v1::ListDirectoryContentsRequest,
                    >::from_parts(&req, &body);
                    svc.list_directory_contents(ctx, req)
                        .await?
                        .encode::<
                            crate::generated::buffa::portal::files::v1::ListDirectoryContentsResponse,
                        >(format)
                })
            }
            "GetDirectoryMetadata" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::generated::buffa::portal::files::v1::GetDirectoryMetadataRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::generated::buffa::portal::files::v1::__buffa::view::GetDirectoryMetadataRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::generated::buffa::portal::files::v1::GetDirectoryMetadataRequest,
                    >::from_parts(&req, &body);
                    svc.get_directory_metadata(ctx, req)
                        .await?
                        .encode::<
                            crate::generated::buffa::portal::files::v1::DirectoryMetadata,
                        >(format)
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_server_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::buffa::bytes::Bytes,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("portal.files.v1.FilesService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "DownloadFile" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::generated::buffa::portal::files::v1::DownloadFileRequest,
                    >(request, format)?;
                    let req: crate::generated::buffa::portal::files::v1::__buffa::view::DownloadFileRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::generated::buffa::portal::files::v1::DownloadFileRequest,
                    >::from_parts(&req, &body);
                    let resp = svc.download_file(ctx, req).await?;
                    Ok(
                        resp
                            .map_body(|s| ::connectrpc::dispatcher::codegen::encode_response_stream::<
                                crate::generated::buffa::portal::files::v1::DownloadFileResponse,
                                _,
                                _,
                            >(s, format)),
                    )
                })
            }
            "ListDirectoryStream" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::generated::buffa::portal::files::v1::ListDirectoryStreamRequest,
                    >(request, format)?;
                    let req: crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryStreamRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::generated::buffa::portal::files::v1::ListDirectoryStreamRequest,
                    >::from_parts(&req, &body);
                    let resp = svc.list_directory_stream(ctx, req).await?;
                    Ok(
                        resp
                            .map_body(|s| ::connectrpc::dispatcher::codegen::encode_response_stream::<
                                crate::generated::buffa::portal::files::v1::DirectoryEntry,
                                _,
                                _,
                            >(s, format)),
                    )
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
    fn call_client_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("portal.files.v1.FilesService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            "UploadFile" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let req_stream = ::connectrpc::dispatcher::codegen::decode_message_request_stream::<
                        crate::generated::buffa::portal::files::v1::UploadFileRequest,
                    >(requests, format);
                    svc.upload_file(ctx, req_stream)
                        .await?
                        .encode::<
                            crate::generated::buffa::portal::files::v1::UploadFileResponse,
                        >(format)
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_bidi_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("portal.files.v1.FilesService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
}
/// Client for this service.
///
/// Generic over `T: ClientTransport`. For **gRPC** (HTTP/2), use
/// `Http2Connection` — it has honest `poll_ready` and composes with
/// `tower::balance` for multi-connection load balancing. For **Connect
/// over HTTP/1.1** (or unknown protocol), use `HttpClient`.
///
/// # Example (gRPC / HTTP/2)
///
/// ```rust,ignore
/// use connectrpc::client::{Http2Connection, ClientConfig};
/// use connectrpc::Protocol;
///
/// let uri: http::Uri = "http://localhost:8080".parse()?;
/// let conn = Http2Connection::connect_plaintext(uri.clone()).await?.shared(1024);
/// let config = ClientConfig::new(uri).with_protocol(Protocol::Grpc);
///
/// let client = FilesServiceClient::new(conn, config);
/// let response = client.upload_file(request).await?;
/// ```
///
/// # Example (Connect / HTTP/1.1 or ALPN)
///
/// ```rust,ignore
/// use connectrpc::client::{HttpClient, ClientConfig};
///
/// let http = HttpClient::plaintext();  // cleartext http:// only
/// let config = ClientConfig::new("http://localhost:8080".parse()?);
///
/// let client = FilesServiceClient::new(http, config);
/// let response = client.upload_file(request).await?;
/// ```
///
/// # Working with the response
///
/// Unary calls return [`UnaryResponse<OwnedView<FooView>>`](::connectrpc::client::UnaryResponse).
/// [`view()`](::connectrpc::client::UnaryResponse::view) borrows the response
/// message, so field access is zero-copy:
///
/// ```rust,ignore
/// let resp = client.upload_file(request).await?;
/// let name: &str = resp.view().name;  // borrow into the response buffer
/// ```
///
/// If you need the owned struct (e.g. to store or pass by value), use
/// [`into_owned()`](::connectrpc::client::UnaryResponse::into_owned):
///
/// ```rust,ignore
/// let owned = client.upload_file(request).await?.into_owned();
/// ```
///
/// [`into_view()`](::connectrpc::client::UnaryResponse::into_view) keeps the
/// zero-copy decoded body (an `OwnedView`) without copying; field access on it
/// goes through `.reborrow()`. Streaming responses yield one `OwnedView` per
/// received message from `.message().await` — bind `msg.reborrow()` for field
/// access, or convert with `.to_owned_message()`.
#[derive(Clone)]
pub struct FilesServiceClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
impl<T> FilesServiceClient<T>
where
    T: ::connectrpc::client::ClientTransport,
    <T::ResponseBody as ::http_body::Body>::Error: ::std::fmt::Display,
{
    /// Create a new client with the given transport and configuration.
    pub fn new(transport: T, config: ::connectrpc::client::ClientConfig) -> Self {
        Self { transport, config }
    }
    /// Get the client configuration.
    pub fn config(&self) -> &::connectrpc::client::ClientConfig {
        &self.config
    }
    /// Get a mutable reference to the client configuration.
    pub fn config_mut(&mut self) -> &mut ::connectrpc::client::ClientConfig {
        &mut self.config
    }
    /// Call the UploadFile RPC. Sends a request to /portal.files.v1.FilesService/UploadFile.
    pub async fn upload_file(
        &self,
        requests: impl IntoIterator<
            Item = crate::generated::buffa::portal::files::v1::UploadFileRequest,
        >,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::generated::buffa::portal::files::v1::__buffa::view::UploadFileResponseView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.upload_file_with_options(
                requests,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the UploadFile RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn upload_file_with_options(
        &self,
        requests: impl IntoIterator<
            Item = crate::generated::buffa::portal::files::v1::UploadFileRequest,
        >,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::generated::buffa::portal::files::v1::__buffa::view::UploadFileResponseView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_client_stream(
                &self.transport,
                &self.config,
                FILES_SERVICE_SERVICE_NAME,
                "UploadFile",
                requests,
                options,
            )
            .await
    }
    /// Call the DownloadFile RPC. Sends a request to /portal.files.v1.FilesService/DownloadFile.
    pub async fn download_file(
        &self,
        request: crate::generated::buffa::portal::files::v1::DownloadFileRequest,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::generated::buffa::portal::files::v1::__buffa::view::DownloadFileResponseView<
                'static,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.download_file_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the DownloadFile RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn download_file_with_options(
        &self,
        request: crate::generated::buffa::portal::files::v1::DownloadFileRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::generated::buffa::portal::files::v1::__buffa::view::DownloadFileResponseView<
                'static,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_server_stream(
                &self.transport,
                &self.config,
                FILES_SERVICE_SERVICE_NAME,
                "DownloadFile",
                request,
                options,
            )
            .await
    }
    /// Call the DeleteFile RPC. Sends a request to /portal.files.v1.FilesService/DeleteFile.
    pub async fn delete_file(
        &self,
        request: crate::generated::buffa::portal::files::v1::DeleteFileRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.delete_file_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the DeleteFile RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn delete_file_with_options(
        &self,
        request: crate::generated::buffa::portal::files::v1::DeleteFileRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                FILES_SERVICE_SERVICE_NAME,
                "DeleteFile",
                request,
                options,
            )
            .await
    }
    /// Call the GetFileMetadata RPC. Sends a request to /portal.files.v1.FilesService/GetFileMetadata.
    pub async fn get_file_metadata(
        &self,
        request: crate::generated::buffa::portal::files::v1::GetFileMetadataRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::generated::buffa::portal::files::v1::__buffa::view::FileMetadataView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.get_file_metadata_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the GetFileMetadata RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn get_file_metadata_with_options(
        &self,
        request: crate::generated::buffa::portal::files::v1::GetFileMetadataRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::generated::buffa::portal::files::v1::__buffa::view::FileMetadataView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                FILES_SERVICE_SERVICE_NAME,
                "GetFileMetadata",
                request,
                options,
            )
            .await
    }
    /// Call the CreateDirectory RPC. Sends a request to /portal.files.v1.FilesService/CreateDirectory.
    pub async fn create_directory(
        &self,
        request: crate::generated::buffa::portal::files::v1::CreateDirectoryRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryMetadataView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.create_directory_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the CreateDirectory RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn create_directory_with_options(
        &self,
        request: crate::generated::buffa::portal::files::v1::CreateDirectoryRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryMetadataView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                FILES_SERVICE_SERVICE_NAME,
                "CreateDirectory",
                request,
                options,
            )
            .await
    }
    /// Call the DeleteDirectory RPC. Sends a request to /portal.files.v1.FilesService/DeleteDirectory.
    pub async fn delete_directory(
        &self,
        request: crate::generated::buffa::portal::files::v1::DeleteDirectoryRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.delete_directory_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the DeleteDirectory RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn delete_directory_with_options(
        &self,
        request: crate::generated::buffa::portal::files::v1::DeleteDirectoryRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                ::buffa_types::google::protobuf::__buffa::view::EmptyView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                FILES_SERVICE_SERVICE_NAME,
                "DeleteDirectory",
                request,
                options,
            )
            .await
    }
    /// Call the ListDirectoryContents RPC. Sends a request to /portal.files.v1.FilesService/ListDirectoryContents.
    pub async fn list_directory_contents(
        &self,
        request: crate::generated::buffa::portal::files::v1::ListDirectoryContentsRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryContentsResponseView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.list_directory_contents_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the ListDirectoryContents RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn list_directory_contents_with_options(
        &self,
        request: crate::generated::buffa::portal::files::v1::ListDirectoryContentsRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::generated::buffa::portal::files::v1::__buffa::view::ListDirectoryContentsResponseView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                FILES_SERVICE_SERVICE_NAME,
                "ListDirectoryContents",
                request,
                options,
            )
            .await
    }
    /// Call the ListDirectoryStream RPC. Sends a request to /portal.files.v1.FilesService/ListDirectoryStream.
    pub async fn list_directory_stream(
        &self,
        request: crate::generated::buffa::portal::files::v1::ListDirectoryStreamRequest,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryEntryView<
                'static,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.list_directory_stream_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the ListDirectoryStream RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn list_directory_stream_with_options(
        &self,
        request: crate::generated::buffa::portal::files::v1::ListDirectoryStreamRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryEntryView<
                'static,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_server_stream(
                &self.transport,
                &self.config,
                FILES_SERVICE_SERVICE_NAME,
                "ListDirectoryStream",
                request,
                options,
            )
            .await
    }
    /// Call the GetDirectoryMetadata RPC. Sends a request to /portal.files.v1.FilesService/GetDirectoryMetadata.
    pub async fn get_directory_metadata(
        &self,
        request: crate::generated::buffa::portal::files::v1::GetDirectoryMetadataRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryMetadataView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.get_directory_metadata_with_options(
                request,
                ::connectrpc::client::CallOptions::default(),
            )
            .await
    }
    /// Call the GetDirectoryMetadata RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn get_directory_metadata_with_options(
        &self,
        request: crate::generated::buffa::portal::files::v1::GetDirectoryMetadataRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::generated::buffa::portal::files::v1::__buffa::view::DirectoryMetadataView<
                    'static,
                >,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                FILES_SERVICE_SERVICE_NAME,
                "GetDirectoryMetadata",
                request,
                options,
            )
            .await
    }
}
