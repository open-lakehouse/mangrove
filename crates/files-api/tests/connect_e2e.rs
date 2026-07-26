//! End-to-end test: start the Files API ConnectRPC server on an ephemeral port
//! over an in-memory store and drive it with the generated client, covering the
//! streaming upload/download round-trip and recursive directory streaming.

use connectrpc::client::{ClientConfig, HttpClient};
use unitycatalog_files_api::testing::spawn_memory_server;
use unitycatalog_files_proto::buffa::portal::files::v1::{
    DownloadFileRequest, GetFileMetadataRequest, ListDirectoryStreamRequest, UploadFileRequest,
};
use unitycatalog_files_proto::connect::portal::files::v1::FilesServiceClient;

fn files_client(base: &str) -> FilesServiceClient<HttpClient> {
    FilesServiceClient::new(
        HttpClient::plaintext(),
        ClientConfig::new(base.parse().unwrap()),
    )
}

#[tokio::test]
async fn streaming_upload_then_download() {
    let base = spawn_memory_server().await;
    let client = files_client(&base);

    let body = b"hello connect streaming world".to_vec();

    // Client-streaming upload: first message carries the path, then the chunks.
    let requests = vec![
        UploadFileRequest {
            path: "/data/hello.txt".into(),
            content_type: Some("text/plain".into()),
            ..Default::default()
        },
        UploadFileRequest {
            chunk: body[..10].to_vec(),
            ..Default::default()
        },
        UploadFileRequest {
            chunk: body[10..].to_vec(),
            ..Default::default()
        },
    ];
    let upload = client.upload_file(requests).await.unwrap().into_owned();
    assert_eq!(upload.path, "/data/hello.txt");
    assert_eq!(upload.file_size, body.len() as i64);

    // Metadata reflects the upload.
    let meta = client
        .get_file_metadata(GetFileMetadataRequest {
            path: "/data/hello.txt".into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    assert_eq!(meta.content_type, "text/plain");
    assert_eq!(meta.file_size, body.len() as i64);

    // Server-streaming download: reassemble the chunks.
    let mut stream = client
        .download_file(DownloadFileRequest {
            path: "/data/hello.txt".into(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut downloaded = Vec::new();
    while let Some(msg) = stream.message().await.unwrap() {
        downloaded.extend_from_slice(&msg.to_owned_message().chunk);
    }
    assert_eq!(downloaded, body);
}

#[tokio::test]
async fn stream_directory_lists_entries() {
    let base = spawn_memory_server().await;
    let client = files_client(&base);

    // Upload a few files under a common directory.
    for name in ["a.txt", "b.txt", "sub/c.txt"] {
        let requests = vec![
            UploadFileRequest {
                path: format!("/data/dir/{name}"),
                ..Default::default()
            },
            UploadFileRequest {
                chunk: b"x".to_vec(),
                ..Default::default()
            },
        ];
        client.upload_file(requests).await.unwrap();
    }

    // Server-streaming recursive list: collect every entry beneath the dir.
    let mut stream = client
        .list_directory_stream(ListDirectoryStreamRequest {
            path: "/data/dir".into(),
            recursive: true,
            ..Default::default()
        })
        .await
        .unwrap();

    let mut paths = Vec::new();
    while let Some(msg) = stream.message().await.unwrap() {
        paths.push(msg.to_owned_message().path);
    }
    paths.sort();
    assert_eq!(
        paths,
        vec![
            "/data/dir/a.txt".to_string(),
            "/data/dir/b.txt".to_string(),
            "/data/dir/sub/c.txt".to_string(),
        ]
    );
}
