use unitycatalog_client::UnityCatalogClient;

// [snippet:list_registered_models]
pub async fn list_registered_models_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let response = client
        .list_registered_models()
        .with_catalog_name("my_catalog".to_string())
        .with_schema_name("my_schema".to_string())
        .await
        .unwrap();
    for model in response.registered_models {
        println!("{}", model.name);
    }
}
// [/snippet:list_registered_models]

// [snippet:create_registered_model]
pub async fn create_registered_model_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let model = client
        .create_registered_model("my_model", "my_catalog", "my_schema")
        .with_comment("My first model".to_string())
        .await
        .unwrap();
    println!("Created: {}", model.full_name);
}
// [/snippet:create_registered_model]

// [snippet:get_registered_model]
pub async fn get_registered_model_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let model = client
        .registered_model("my_catalog", "my_schema", "my_model")
        .get()
        .await
        .unwrap();
    println!("Got: {}", model.name);
}
// [/snippet:get_registered_model]

// [snippet:create_model_version]
pub async fn create_model_version_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    // A new version starts in PENDING_REGISTRATION. Write your artifacts to the
    // returned `storage_location` (vending credentials as needed), then finalize.
    let version = client
        .create_model_version(
            "my_model",
            "my_catalog",
            "my_schema",
            "s3://my-run/artifacts",
        )
        .await
        .unwrap();
    println!("Created version {}", version.version);
}
// [/snippet:create_model_version]

// [snippet:finalize_model_version]
pub async fn finalize_model_version_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    // Once all artifacts are written, finalize to transition the version to READY.
    let version = client
        .finalize_model_version("my_catalog.my_schema.my_model", 1)
        .await
        .unwrap();
    println!(
        "Finalized version {} -> {:?}",
        version.version, version.status
    );
}
// [/snippet:finalize_model_version]

// [snippet:list_model_versions]
pub async fn list_model_versions_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let response = client
        .list_model_versions("my_catalog.my_schema.my_model")
        .await
        .unwrap();
    for version in response.model_versions {
        println!("version {}", version.version);
    }
}
// [/snippet:list_model_versions]
