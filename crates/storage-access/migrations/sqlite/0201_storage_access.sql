-- Storage credentials and the external locations they govern (SQLite).
--
-- Owned by `SqliteStorageAccess`, apart from the generic object graph: the
-- catalog resolves a storage path to its governing location on every credential
-- vend, so locations are matched on an indexed canonical key rather than by
-- listing them all.
--
--   * Names are ASCII case-insensitive (`COLLATE NOCASE`), like object names.
--   * `url_key` is the canonical location with a trailing `/` (`LocationKey`), so
--     covering is a string prefix: the keys covering a path are an `IN` list of
--     its ancestors, the keys under a path a range scan. Both use the unique
--     index under the default bytewise collation.
--   * `sealed` is the envelope-encrypted secret configuration, bound to the
--     credential id.
--   * Timestamps are INTEGER epoch millis (the wire representation).

CREATE TABLE uc_credentials (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
    purpose INTEGER NOT NULL,
    read_only INTEGER NOT NULL DEFAULT 0,
    used_for_managed_storage INTEGER NOT NULL DEFAULT 0,
    comment TEXT,
    owner TEXT,
    sealed BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    created_by TEXT,
    updated_at INTEGER NOT NULL,
    updated_by TEXT
);

CREATE INDEX uc_credentials_purpose ON uc_credentials (purpose, name);

CREATE TABLE uc_external_locations (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
    url TEXT NOT NULL,
    url_key TEXT NOT NULL UNIQUE,
    credential_id TEXT NOT NULL REFERENCES uc_credentials (id) ON DELETE RESTRICT,
    read_only INTEGER NOT NULL DEFAULT 0,
    comment TEXT,
    owner TEXT,
    created_at INTEGER NOT NULL,
    created_by TEXT,
    updated_at INTEGER NOT NULL,
    updated_by TEXT
);

CREATE INDEX uc_external_locations_credential ON uc_external_locations (credential_id);
