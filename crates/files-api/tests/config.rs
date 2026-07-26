//! Integration coverage for the standalone binary's YAML config loading.

use std::io::Write;

use unitycatalog_files_api::cli::config::{AuthKind, Backend, Config};

/// Write `contents` to a temp file and return its path (kept alive by the caller
/// holding the returned `NamedTempFile`).
fn temp_yaml(contents: &str) -> (tempfile::NamedTempFile, String) {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(contents.as_bytes()).unwrap();
    let path = f.path().to_string_lossy().into_owned();
    (f, path)
}

#[test]
fn loads_unity_config_from_file() {
    let (_f, path) = temp_yaml(
        r#"
        host: "127.0.0.1"
        port: 9111
        base_path: "/files"
        upstream:
          base_url: "https://uc.example/api/2.1/unity-catalog/"
          token:
            env: "UC_TOKEN"
        "#,
    );
    let cfg = Config::load(&path).unwrap();
    assert_eq!(cfg.resolved_host(), "127.0.0.1");
    assert_eq!(cfg.resolved_port(), 9111);
    assert_eq!(cfg.resolved_base_path(), "/files");
    assert_eq!(cfg.backend, Backend::Unity);
    assert_eq!(cfg.auth.mode, AuthKind::Anonymous);
    cfg.validate().unwrap();
}

#[test]
fn loads_memory_config_from_file() {
    let (_f, path) = temp_yaml("backend: memory\n");
    let cfg = Config::load(&path).unwrap();
    assert_eq!(cfg.backend, Backend::Memory);
    assert!(cfg.upstream.is_none());
    cfg.validate().unwrap();
}

#[test]
fn missing_file_is_an_error() {
    assert!(Config::load("/no/such/files-api-config.yaml").is_err());
}
