import { UnityCatalogClient } from "@unitycatalog/client";

// [snippet:list_registered_models]
export async function listRegisteredModelsExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const models = await client.listRegisteredModels({
    catalogName: "my_catalog",
    schemaName: "my_schema",
  });
  for (const model of models) {
    console.log(model.name);
  }
}
// [/snippet:list_registered_models]

// [snippet:create_registered_model]
export async function createRegisteredModelExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const model = await client.createRegisteredModel(
    "my_model",
    "my_catalog",
    "my_schema",
    { comment: "My first model" },
  );
  console.log(`Created: ${model.fullName}`);
}
// [/snippet:create_registered_model]

// [snippet:get_registered_model]
export async function getRegisteredModelExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const model = await client
    .registeredModel("my_catalog", "my_schema", "my_model")
    .get();
  console.log(`Got: ${model.name}`);
}
// [/snippet:get_registered_model]

// [snippet:create_model_version]
export async function createModelVersionExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  // A new version starts in PENDING_REGISTRATION. Write your artifacts to the
  // returned storageLocation (vending credentials as needed), then finalize.
  const version = await client.createModelVersion(
    "my_model",
    "my_catalog",
    "my_schema",
    "s3://my-run/artifacts",
  );
  console.log(`Created version ${version.version}`);
}
// [/snippet:create_model_version]

// [snippet:finalize_model_version]
export async function finalizeModelVersionExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  // Once all artifacts are written, finalize to transition the version to READY.
  const version = await client.finalizeModelVersion(
    "my_catalog.my_schema.my_model",
    1,
  );
  console.log(`Finalized version ${version.version}`);
}
// [/snippet:finalize_model_version]

// [snippet:list_model_versions]
export async function listModelVersionsExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const versions = await client.listModelVersions(
    "my_catalog.my_schema.my_model",
  );
  for (const version of versions) {
    console.log(`version ${version.version}`);
  }
}
// [/snippet:list_model_versions]
