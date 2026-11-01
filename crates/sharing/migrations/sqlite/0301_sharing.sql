-- Shares, recipients, their bearer tokens and the grants between them (SQLite).
--
-- Owned by `SqliteSharingStore`. The same tables as the Postgres schema:
-- secrets are stored sealed (envelope encryption), recipient tokens and
-- activation codes only as sha256 digests. Deletes cascade through foreign
-- keys, which the host's connections must enforce (`PRAGMA foreign_keys = ON`).

CREATE TABLE uc_shares (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    comment     TEXT,
    -- The share's objects (UC `DataObject`s) as a JSON array, in the order
    -- they were added.
    objects     TEXT NOT NULL DEFAULT '[]',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE uc_recipients (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    comment     TEXT,
    created_at  INTEGER NOT NULL
);

CREATE TABLE uc_recipient_tokens (
    id                    TEXT PRIMARY KEY,
    recipient_id          TEXT NOT NULL REFERENCES uc_recipients (id) ON DELETE CASCADE,
    token_hash            BLOB NOT NULL UNIQUE,
    -- Kept only until the recipient activates the token.
    sealed_token          BLOB,
    -- The sha256 digest of the activation code, which is derived from the
    -- token; the link is rebuilt from the sealed token while that is kept.
    activation_code_hash  BLOB UNIQUE,
    expires_at            INTEGER,
    created_at            INTEGER NOT NULL
);

CREATE INDEX uc_recipient_tokens_recipient ON uc_recipient_tokens (recipient_id);

CREATE TABLE uc_share_grants (
    share_id      TEXT NOT NULL REFERENCES uc_shares (id) ON DELETE CASCADE,
    recipient_id  TEXT NOT NULL REFERENCES uc_recipients (id) ON DELETE CASCADE,
    PRIMARY KEY (share_id, recipient_id)
);

CREATE INDEX uc_share_grants_recipient ON uc_share_grants (recipient_id);
