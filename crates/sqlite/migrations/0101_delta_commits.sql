-- Ratified-but-unpublished Delta catalog-managed commits (SQLite).
--
-- SQLite translation of the Postgres `delta_commits` table. The unique
-- constraint on (table_id, commit_version) is the first-writer-wins arbiter:
-- when two writers race the same version, exactly one insert succeeds and the
-- other surfaces a unique violation (mapped to a version conflict).
--
-- Dialect notes (consistent with the rest of the schema):
--   * `id` / `table_id` are BLOBs holding the 16 raw bytes of a UUID.
--   * version / file size are INTEGER; timestamps are INTEGER epoch millis (the
--     wire representation supplied by the Delta client).
--   * `is_backfilled_latest` is an INTEGER boolean (0/1).
--   * `updated_at` is maintained by the application (no triggers).

CREATE TABLE delta_commits (
    id BLOB PRIMARY KEY NOT NULL,
    table_id BLOB NOT NULL,
    commit_version INTEGER NOT NULL,
    commit_filename TEXT NOT NULL,
    commit_filesize INTEGER NOT NULL,
    commit_file_modification_timestamp INTEGER NOT NULL,
    -- The in-commit timestamp supplied by the Delta client.
    commit_timestamp INTEGER NOT NULL,
    -- Set on the highest commit once it has been fully backfilled: the row is
    -- retained as a version marker but hidden from `get_commits`.
    is_backfilled_latest INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    CONSTRAINT unique_delta_commit UNIQUE (table_id, commit_version)
);

CREATE INDEX delta_commits_table_index ON delta_commits (table_id, commit_version);
