-- Storage credentials and the external locations they govern (Postgres).
--
-- Owned by `PgStorageAccess`, apart from the generic object graph: the catalog
-- resolves a storage path to its governing location on every credential vend,
-- so locations are matched on an indexed canonical key rather than by listing
-- them all.
--
--   * Names are case-insensitive through the `case_insensitive` ICU collation,
--     like object names (re-declared here so this schema stands on its own).
--   * `url_key` is the canonical location with a trailing `/` (`LocationKey`), so
--     covering is a string prefix: the keys covering a path are an `= ANY` lookup
--     over its ancestors, the keys under a path a range scan. Both use the unique
--     index; the column is `COLLATE "C"` so the range is bytewise.
--   * `sealed` is the envelope-encrypted secret configuration, bound to the
--     credential id.
--   * Timestamps are bigint epoch millis (the wire representation).

CREATE COLLATION IF NOT EXISTS case_insensitive (
    provider = icu,
    locale = 'und-u-ks-level2',
    deterministic = false
);

CREATE TABLE uc_credentials (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL COLLATE case_insensitive UNIQUE,
    purpose integer NOT NULL,
    read_only boolean NOT NULL DEFAULT false,
    used_for_managed_storage boolean NOT NULL DEFAULT false,
    comment text,
    owner text,
    sealed bytea NOT NULL,
    created_at bigint NOT NULL,
    created_by text,
    updated_at bigint NOT NULL,
    updated_by text
);

CREATE INDEX uc_credentials_purpose ON uc_credentials (purpose, name);

CREATE TABLE uc_external_locations (
    id uuid PRIMARY KEY NOT NULL,
    name text NOT NULL COLLATE case_insensitive UNIQUE,
    url text NOT NULL,
    url_key text NOT NULL COLLATE "C" UNIQUE,
    credential_id uuid NOT NULL REFERENCES uc_credentials (id) ON DELETE RESTRICT,
    read_only boolean NOT NULL DEFAULT false,
    comment text,
    owner text,
    created_at bigint NOT NULL,
    created_by text,
    updated_at bigint NOT NULL,
    updated_by text
);

CREATE INDEX uc_external_locations_credential ON uc_external_locations (credential_id);
