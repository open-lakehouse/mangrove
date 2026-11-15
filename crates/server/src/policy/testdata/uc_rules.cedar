// A small Unity Catalog model over the server's vocabulary, for the tests: the
// privileges of the grants on each securable (`privileges`) and inherited down
// the hierarchy (`inherited_privileges`), with `OWNER` for ownership.
//
// As in luminescent-lakehouse's policies: owning a securable counts as every
// privilege on it but EXTERNAL USE, while owning a container gives no data
// access to what is in it (only the use of the container itself);
// `ALL_PRIVILEGES` covers neither MANAGE nor EXTERNAL USE.

// `admin` may do anything.
@id("admin")
permit (principal == Lakehouse::User::"admin", action, resource);

// Anyone may list the catalogs; each one is then filtered by `GetCatalog`.
@id("list-catalogs")
permit (
  principal,
  action == UnityCatalog::Action::"ListCatalogs",
  resource == UnityCatalog::Metastore::"metastore"
);

@id("get-catalog")
permit (principal, action == UnityCatalog::Action::"GetCatalog", resource is UnityCatalog::Catalog)
when {
  resource.privileges.contains("OWNER") ||
  resource.inherited_privileges.containsAny(["USE_CATALOG", "BROWSE", "ALL_PRIVILEGES", "MANAGE"])
};

// Lists in a schema need USE on both containers.
@id("list-tables")
permit (principal, action == UnityCatalog::Action::"ListTables", resource is UnityCatalog::Schema)
when {
  (resource.catalog.privileges.contains("OWNER") ||
    resource.catalog.inherited_privileges.containsAny(["USE_CATALOG", "ALL_PRIVILEGES"])) &&
  (resource.privileges.contains("OWNER") ||
    resource.inherited_privileges.containsAny(["USE_SCHEMA", "ALL_PRIVILEGES"]))
};

// Seeing a table: USE on its containers and any privilege on it.
@id("get-table")
permit (
  principal,
  action in [UnityCatalog::Action::"GetTable", UnityCatalog::Action::"GetTableExists"],
  resource is UnityCatalog::Table
)
when {
  (resource.catalog.privileges.contains("OWNER") ||
    resource.catalog.inherited_privileges.containsAny(["USE_CATALOG", "ALL_PRIVILEGES"])) &&
  (resource.schema.privileges.contains("OWNER") ||
    resource.schema.inherited_privileges.containsAny(["USE_SCHEMA", "ALL_PRIVILEGES"])) &&
  (resource.privileges.contains("OWNER") ||
    resource.inherited_privileges.containsAny(["SELECT", "MODIFY", "BROWSE", "MANAGE", "ALL_PRIVILEGES"]))
};

@id("create-table")
permit (principal, action == UnityCatalog::Action::"CreateTable", resource is UnityCatalog::Table)
when {
  (resource.catalog.privileges.contains("OWNER") ||
    resource.catalog.inherited_privileges.containsAny(["USE_CATALOG", "ALL_PRIVILEGES"])) &&
  (resource.schema.privileges.contains("OWNER") ||
    (resource.schema.inherited_privileges.containsAny(["USE_SCHEMA", "ALL_PRIVILEGES"]) &&
      resource.schema.inherited_privileges.containsAny(["CREATE_TABLE", "ALL_PRIVILEGES"])))
};

// Deleting or renaming a table is its owner's (or a manager's).
@id("drop-table")
permit (
  principal,
  action in [UnityCatalog::Action::"DeleteTable", UnityCatalog::Action::"RenameTable"],
  resource is UnityCatalog::Table
)
when {
  (resource.catalog.privileges.contains("OWNER") ||
    resource.catalog.inherited_privileges.containsAny(["USE_CATALOG", "ALL_PRIVILEGES"])) &&
  (resource.schema.privileges.contains("OWNER") ||
    resource.schema.inherited_privileges.containsAny(["USE_SCHEMA", "ALL_PRIVILEGES"])) &&
  (resource.privileges.contains("OWNER") || resource.inherited_privileges.contains("MANAGE"))
};

@id("commit-table")
permit (principal, action == UnityCatalog::Action::"CommitTable", resource is UnityCatalog::Table)
when {
  (resource.catalog.privileges.contains("OWNER") ||
    resource.catalog.inherited_privileges.containsAny(["USE_CATALOG", "ALL_PRIVILEGES"])) &&
  (resource.schema.privileges.contains("OWNER") ||
    resource.schema.inherited_privileges.containsAny(["USE_SCHEMA", "ALL_PRIVILEGES"])) &&
  (resource.privileges.contains("OWNER") ||
    resource.inherited_privileges.containsAny(["MODIFY", "ALL_PRIVILEGES"]))
};

// Table credentials: SELECT (and MODIFY to write), plus EXTERNAL USE SCHEMA,
// which nothing else implies. Every vend says which access it asks for.
@id("vend-table")
permit (
  principal,
  action == UnityCatalog::Action::"GenerateTemporaryTableCredentials",
  resource is UnityCatalog::Table
)
when {
  context has credential_operation &&
  (resource.catalog.privileges.contains("OWNER") ||
    resource.catalog.inherited_privileges.containsAny(["USE_CATALOG", "ALL_PRIVILEGES"])) &&
  (resource.schema.privileges.contains("OWNER") ||
    resource.schema.inherited_privileges.containsAny(["USE_SCHEMA", "ALL_PRIVILEGES"])) &&
  resource.schema.inherited_privileges.contains("EXTERNAL_USE_SCHEMA") &&
  (resource.privileges.contains("OWNER") ||
    resource.inherited_privileges.containsAny(["SELECT", "ALL_PRIVILEGES"])) &&
  (context.credential_operation == "READ" ||
    resource.privileges.contains("OWNER") ||
    resource.inherited_privileges.containsAny(["MODIFY", "ALL_PRIVILEGES"]))
};

// Grants are their securable's owner's (or a manager's).
@id("grants")
permit (
  principal,
  action in [UnityCatalog::Action::"GetGrants", UnityCatalog::Action::"UpdateGrants"],
  resource
)
when { resource.privileges.contains("OWNER") || resource.inherited_privileges.contains("MANAGE") };
