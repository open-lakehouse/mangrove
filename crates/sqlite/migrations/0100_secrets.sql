-- Mangrove-local secrets table (SQLite).
--
-- The `objects` / `associations` graph tables are owned and migrated by the
-- generic `olai_store::SqlStore` (see `olai_store::migrate_sql`); this crate's
-- migrator carries only the mangrove-specific tables that share the same DB.
-- The high version prefix (0100+) keeps these clear of the generic store's
-- migration versions (0001+), so both migrators coexist on one database.
--
-- Dialect notes:
--   * `id` is a BLOB holding the 16 raw bytes of a UUIDv7 (generated Rust-side).
--   * `COLLATE NOCASE` gives ASCII case-insensitive name matching.
--   * Timestamps are INTEGER microseconds since the Unix epoch (UTC);
--     `updated_at` is maintained explicitly by the application (no triggers).

CREATE TABLE secrets (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    value BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER
);
