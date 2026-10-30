import {
  CatalogClient,
  CredentialClient,
  ExternalLocationClient,
  SchemaClient,
  TableClient,
  UnityCatalogClient,
  VolumeClient,
} from "../dist/index";

const BASE_URL = "http://unitycatalog.io/api/2.1/unity-catalog/";

describe("UnityCatalogClient", () => {
  let client: UnityCatalogClient;

  beforeEach(() => {
    client = new UnityCatalogClient(BASE_URL);
  });

  it("should create a new instance", () => {
    expect(client).toBeDefined();
  });

  it("should accept an optional token", () => {
    const authed = new UnityCatalogClient(BASE_URL, "my-token");
    expect(authed).toBeDefined();
  });

  describe("collection methods exist", () => {
    it("has listCatalogs", () => {
      expect(client.listCatalogs).toBeDefined();
    });

    it("has createCatalog", () => {
      expect(client.createCatalog).toBeDefined();
    });

    it("has listSchemas", () => {
      expect(client.listSchemas).toBeDefined();
    });

    it("has createSchema", () => {
      expect(client.createSchema).toBeDefined();
    });

    it("has listTables", () => {
      expect(client.listTables).toBeDefined();
    });

    it("has createTable", () => {
      expect(client.createTable).toBeDefined();
    });

    it("has listCredentials", () => {
      expect(client.listCredentials).toBeDefined();
    });

    it("has createCredential", () => {
      expect(client.createCredential).toBeDefined();
    });

    it("has listExternalLocations", () => {
      expect(client.listExternalLocations).toBeDefined();
    });

    it("has createExternalLocation", () => {
      expect(client.createExternalLocation).toBeDefined();
    });

    it("has listVolumes", () => {
      expect(client.listVolumes).toBeDefined();
    });

    it("has createVolume", () => {
      expect(client.createVolume).toBeDefined();
    });
  });

  describe("resource accessors", () => {
    it("returns a CatalogClient", () => {
      const cat = client.catalog("my-catalog");
      expect(cat).toBeInstanceOf(CatalogClient);
    });

    it("returns a SchemaClient", () => {
      const schema = client.schema("my-catalog", "my-schema");
      expect(schema).toBeInstanceOf(SchemaClient);
    });

    it("returns a CredentialClient", () => {
      const cred = client.credential("my-cred");
      expect(cred).toBeInstanceOf(CredentialClient);
    });

    it("returns an ExternalLocationClient", () => {
      const loc = client.externalLocation("my-loc");
      expect(loc).toBeInstanceOf(ExternalLocationClient);
    });

    it("returns a VolumeClient", () => {
      const vol = client.volume("my-catalog", "my-schema", "my-volume");
      expect(vol).toBeInstanceOf(VolumeClient);
    });

    it("returns a TableClient", () => {
      const table = client.table("my-catalog", "my-schema", "my-table");
      expect(table).toBeInstanceOf(TableClient);
    });
  });

  describe("resource client methods", () => {
    it("CatalogClient has get, update, delete", () => {
      const cat = client.catalog("my-catalog");
      expect(cat.get).toBeDefined();
      expect(cat.update).toBeDefined();
      expect(cat.delete).toBeDefined();
    });

    it("SchemaClient has get, update, delete", () => {
      const schema = client.schema("my-catalog", "my-schema");
      expect(schema.get).toBeDefined();
      expect(schema.update).toBeDefined();
      expect(schema.delete).toBeDefined();
    });

    it("VolumeClient has get, update, delete", () => {
      const vol = client.volume("cat", "sch", "vol");
      expect(vol.get).toBeDefined();
      expect(vol.update).toBeDefined();
      expect(vol.delete).toBeDefined();
    });

    it("TableClient has get, delete", () => {
      const table = client.table("cat", "sch", "tbl");
      expect(table.get).toBeDefined();
      expect(table.delete).toBeDefined();
    });
  });
});
