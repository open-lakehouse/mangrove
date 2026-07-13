//! Integration test for the Azurite credential-vending path.
//!
//! Proves the full offline-SAS vend path end-to-end against a live Azurite
//! (Azure Blob emulator): the server signs a SAS from a storage-account key
//! (no STS / online token service), and that vended SAS actually authorizes
//! blob I/O at the expected scope. This is the Rust analogue of
//! `open-lakehouse/scripts/azurite-seed.sh`.
//!
//! Gated behind the `integration-azurite` feature and `#[ignore]` so it never
//! runs in a normal `cargo test`. It needs a running Azurite blob emulator on
//! `localhost:10000`. Run it with:
//!
//! ```sh
//! just integration-azurite
//! ```
//!
//! Environment knobs:
//! | Variable                  | Default                  |
//! |---------------------------|--------------------------|
//! | `UC_AZURITE_BLOB_ENDPOINT`| `http://127.0.0.1:10000` |
//! | `UC_AZURITE_CONTAINER`    | `lakehouse`              |
#![cfg(feature = "integration-azurite")]

use std::sync::Arc;

use object_store::azure::{AzureConfigKey, MicrosoftAzureBuilder};
use object_store::{ObjectStore, ObjectStoreExt, PutPayload, path::Path};
use unitycatalog_common::models::credentials::v1::{
    AzureStorageKey, CreateCredentialRequest, Purpose,
};
use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
use unitycatalog_common::models::temporary_credentials::v1::{
    GenerateTemporaryPathCredentialsRequest,
    generate_temporary_path_credentials_request::Operation, temporary_credential::Credentials,
};
use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_server::api::{
    CredentialHandler, ExternalLocationHandler, RequestContext, TemporaryCredentialHandler,
};
use unitycatalog_server::memory::InMemoryResourceStore;
use unitycatalog_server::policy::{ConstantPolicy, Policy, Principal};
use unitycatalog_server::services::ServerHandler;

/// The well-known Azurite account and key (local dev only — published by
/// Microsoft, deliberately not secret).
const ACCOUNT: &str = "devstoreaccount1";
const ACCOUNT_KEY: &str =
    "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";

fn container() -> String {
    std::env::var("UC_AZURITE_CONTAINER").unwrap_or_else(|_| "lakehouse".to_string())
}

fn blob_endpoint() -> String {
    std::env::var("UC_AZURITE_BLOB_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:10000".to_string())
}

fn handler() -> ServerHandler<RequestContext> {
    let encryptor =
        EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
    let store = Arc::new(InMemoryResourceStore::new(encryptor));
    let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
    ServerHandler::try_new_tokio(policy, store).unwrap()
}

fn ctx() -> RequestContext {
    RequestContext {
        recipient: Principal::anonymous(),
    }
}

/// Register the `azure_storage_key` credential + a covering external location at
/// `azurite://<container>`, mirroring `azurite-seed.sh`.
async fn seed(h: &ServerHandler<RequestContext>) {
    h.create_credential(
        CreateCredentialRequest {
            name: "azurite_key".to_string(),
            purpose: Purpose::Storage.into(),
            azure_storage_key: Some(AzureStorageKey {
                account_name: ACCOUNT.to_string(),
                account_key: ACCOUNT_KEY.to_string(),
                ..Default::default()
            })
            .into(),
            // The emulator account is not reachable for online validation here;
            // we prove usability by actually performing blob I/O below.
            skip_validation: Some(true),
            ..Default::default()
        },
        ctx(),
    )
    .await
    .unwrap();
    h.create_external_location(
        CreateExternalLocationRequest {
            name: "azurite_loc".to_string(),
            url: format!("azurite://{}", container()),
            credential_name: "azurite_key".to_string(),
            ..Default::default()
        },
        ctx(),
    )
    .await
    .unwrap();
}

/// Vend a path credential for `url` and return its SAS token.
async fn vend_sas(h: &ServerHandler<RequestContext>, url: &str, operation: Operation) -> String {
    let cred = h
        .generate_temporary_path_credentials(
            GenerateTemporaryPathCredentialsRequest {
                url: url.to_string(),
                operation: operation.into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .expect("vending should succeed");
    match cred.credentials {
        Some(Credentials::AzureUserDelegationSas(sas)) => sas.sas_token,
        other => panic!("expected an Azure SAS credential, got {other:?}"),
    }
}

/// Build an emulator-mode object store addressed at the container root, using a
/// vended SAS — the same construction as the object-store crate's Azurite
/// branch (`to_store`). Blob paths are relative to the container.
fn azurite_store(sas: &str) -> Arc<dyn ObjectStore> {
    // Point the emulator at the configured blob endpoint (`<endpoint>/<account>`),
    // so the test works whether Azurite is on localhost or a CI service hostname.
    let endpoint = format!("{}/{ACCOUNT}", blob_endpoint().trim_end_matches('/'));
    let store = MicrosoftAzureBuilder::new()
        .with_use_emulator(true)
        .with_account(ACCOUNT)
        .with_container_name(container())
        .with_config(AzureConfigKey::Endpoint, endpoint)
        .with_config(AzureConfigKey::SasKey, sas.to_string())
        .build()
        .expect("emulator store builds");
    Arc::new(store)
}

/// A read-write vended SAS authorizes both writes and reads at its scope.
#[tokio::test]
#[ignore = "requires a running Azurite emulator (just integration-azurite)"]
async fn read_write_sas_can_put_and_get() {
    let h = handler();
    seed(&h).await;

    let prefix = "sales/orders";
    let url = format!("azurite://{}/{prefix}", container());
    let sas = vend_sas(&h, &url, Operation::PathReadWrite).await;

    let store = azurite_store(&sas);
    let blob = Path::from(format!("{prefix}/vended.txt"));
    let body = b"written-with-vended-sas";

    store
        .put(&blob, PutPayload::from_static(body))
        .await
        .expect("read-write SAS should authorize PUT");
    let got = store
        .get(&blob)
        .await
        .expect("read-write SAS should authorize GET")
        .bytes()
        .await
        .unwrap();
    assert_eq!(&got[..], body, "round-trip body should match");
}

/// A read-only vended SAS authorizes reads but denies writes.
#[tokio::test]
#[ignore = "requires a running Azurite emulator (just integration-azurite)"]
async fn read_only_sas_can_get_but_not_put() {
    let h = handler();
    seed(&h).await;

    let prefix = "sales/orders";
    let url = format!("azurite://{}/{prefix}", container());

    // First write a blob with a read-write SAS so there is something to read.
    let rw = vend_sas(&h, &url, Operation::PathReadWrite).await;
    let rw_store = azurite_store(&rw);
    let blob = Path::from(format!("{prefix}/ro-probe.txt"));
    rw_store
        .put(&blob, PutPayload::from_static(b"seed"))
        .await
        .expect("setup write should succeed");

    // A read-only SAS can read it back...
    let ro = vend_sas(&h, &url, Operation::PathRead).await;
    let ro_store = azurite_store(&ro);
    let got = ro_store
        .get(&blob)
        .await
        .expect("read-only SAS should authorize GET")
        .bytes()
        .await
        .unwrap();
    assert_eq!(&got[..], b"seed");

    // ...but cannot write.
    let denied = Path::from(format!("{prefix}/ro-denied.txt"));
    let res = ro_store.put(&denied, PutPayload::from_static(b"x")).await;
    assert!(
        res.is_err(),
        "read-only SAS must NOT authorize PUT, but the write succeeded"
    );
}
