-- Ratified-but-unpublished Delta catalog-managed commits (Postgres).
--
-- The object/association graph tables are owned and migrated by the generic
-- `olai_store::PgStore` (see `olai_store::migrate_pg`); this crate's migrator
-- carries only the mangrove-specific tables that share the same database. The
-- high version prefix (0100+) keeps these clear of the generic store's migration
-- versions (0001+), so `pg_migrator_with` merges both into one ordered ledger.
--
-- The unique constraint on (table_id, commit_version) is the first-writer-wins
-- arbiter: when two writers race the same version, exactly one insert succeeds
-- and the other surfaces a unique violation (mapped to a version conflict).
--
-- Self-contained (no custom `uuidv7()` / `trigger_updated_at()` helpers): `id`
-- defaults to the built-in `gen_random_uuid()` and `created_at` to `now()`;
-- `updated_at` is left to the application (backfill never needs it).
CREATE TABLE delta_commits (
    id uuid PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    table_id uuid NOT NULL,
    commit_version bigint NOT NULL,
    commit_filename text NOT NULL,
    commit_filesize bigint NOT NULL,
    commit_file_modification_timestamp timestamptz NOT NULL,
    -- The in-commit timestamp supplied by the Delta client.
    commit_timestamp timestamptz NOT NULL,
    -- Set on the highest commit once it has been fully backfilled: the row is
    -- retained as a version marker but hidden from `get_commits`.
    is_backfilled_latest boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz,
    CONSTRAINT unique_delta_commit UNIQUE (table_id, commit_version)
);

CREATE INDEX delta_commits_table_index ON delta_commits (table_id, commit_version);
