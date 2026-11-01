# olai-uc-storage-access

**Storage credentials and the external locations they govern, behind one store
built for how the catalog uses them: resolving which location, and which
credential, governs a storage path.**

> [!NOTE]
> The Rust crate identifier is `unitycatalog_storage_access` (imports and paths
> use that name); the crate is published to crates.io as `olai-uc-storage-access`
> while the naming settles. This is an experimental component of an unofficial
> [Unity Catalog](https://unitycatalog.io) implementation.

`StorageAccessStore` is the port. It owns the integrity rules of the pair: an
external location must name an existing credential, no two external locations
may overlap, and a credential cannot be deleted while a location uses it. Its
lookups, `governing_location` and `covering_location`, answer "which registered
location covers this path" without listing every location, so a backend can
serve them from an index.

Implementations:

- `InMemoryStorageAccess` (feature `memory`), for tests and ephemeral servers;
- `PgStorageAccess` (feature `postgres`) and `SqliteStorageAccess` (feature
  `sqlite`), over a pool the host provides, each with tables of its own. Their
  `MIGRATOR` (versioned 0201) is meant to be merged into the host's migrator so
  the database keeps one `_sqlx_migrations` ledger. `olai-uc-postgres` and
  `olai-uc-sqlite` do exactly that.

`rewrap_secrets` re-wraps sealed secrets under the active key after a KEK
rotation.

With the `vending` feature, `dyn StorageAccessStore` implements
`olai_credential_vending::CredentialResolver`. The `testing` feature exposes the
conformance suite every backend runs (`testing::conformance`), and
`testing::key_rotation` for backends that seal secrets.
