-- Embedded SQLite schema for the Unity Catalog graph store.
--
-- This is the SQLite translation of the Postgres backend's `objects` /
-- `associations` / `secrets` tables. Notable dialect differences:
--
--   * `id` columns are BLOBs holding the 16 raw bytes of a UUIDv7. v7 is
--     time-ordered, so `ORDER BY id` preserves creation order for keyset
--     pagination (the same property the Postgres backend relies on). UUIDs are
--     always generated Rust-side (no `uuidv7()` SQL function).
--   * `label` / `to_label` are TEXT (snake_case), not native ENUM types. The
--     allowed set lives in the `ObjectLabel` / `AssociationLabel` Rust enums.
--   * `name` / `namespace` are TEXT holding the escaped `ResourceName` encoding
--     (dot-separated, backtick-escaped). `namespace` is the encoded parent path,
--     denormalized so namespace listing is an indexed equality lookup rather
--     than an array slice.
--   * `properties` is TEXT holding serialized JSON.
--   * Timestamps are INTEGER microseconds since the Unix epoch (UTC).
--   * `updated_at` is maintained explicitly by the application on each UPDATE
--     (no triggers).
--   * `COLLATE NOCASE` gives ASCII-only case-insensitive name matching. The
--     Postgres backend uses an ICU `case_insensitive` collation; non-ASCII
--     case folding is therefore a known behavioral gap (see crate docs).

CREATE TABLE objects (
    id BLOB PRIMARY KEY NOT NULL,
    label TEXT NOT NULL,
    name TEXT NOT NULL COLLATE NOCASE,
    namespace TEXT NOT NULL COLLATE NOCASE,
    properties TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    CONSTRAINT unique_object_name UNIQUE (label, name)
);

CREATE INDEX objects_namespace_index ON objects (label, namespace);

CREATE TABLE associations (
    id BLOB PRIMARY KEY NOT NULL,
    from_id BLOB NOT NULL REFERENCES objects (id),
    label TEXT NOT NULL,
    to_id BLOB NOT NULL REFERENCES objects (id),
    to_label TEXT NOT NULL,
    properties TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    CONSTRAINT unique_association UNIQUE (from_id, label, to_id)
);

CREATE INDEX associations_tuple_index ON associations (from_id, label, to_id);

CREATE TABLE secrets (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    value BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER
);
