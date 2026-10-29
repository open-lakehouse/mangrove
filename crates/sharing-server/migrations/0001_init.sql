-- The Open Sharing server's state. Secrets are stored sealed (envelope
-- encryption); recipient tokens only as sha256 digests.

CREATE TABLE sharing_shares (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    comment     TEXT,
    -- The share's UC `DataObject`s, in the order they were added.
    objects     JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at  BIGINT NOT NULL,
    updated_at  BIGINT NOT NULL
);

CREATE TABLE sharing_recipients (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    comment     TEXT,
    created_at  BIGINT NOT NULL
);

CREATE TABLE sharing_recipient_tokens (
    id               TEXT PRIMARY KEY,
    recipient_id     TEXT NOT NULL REFERENCES sharing_recipients (id) ON DELETE CASCADE,
    token_hash       BYTEA NOT NULL UNIQUE,
    -- Kept only until the recipient activates the token.
    sealed_token     BYTEA,
    activation_code  TEXT UNIQUE,
    expires_at       BIGINT,
    created_at       BIGINT NOT NULL
);

CREATE INDEX sharing_recipient_tokens_recipient ON sharing_recipient_tokens (recipient_id);

CREATE TABLE sharing_grants (
    share_id      TEXT NOT NULL REFERENCES sharing_shares (id) ON DELETE CASCADE,
    recipient_id  TEXT NOT NULL REFERENCES sharing_recipients (id) ON DELETE CASCADE,
    PRIMARY KEY (share_id, recipient_id)
);

CREATE INDEX sharing_grants_recipient ON sharing_grants (recipient_id);

-- Local storage access: sealed storage credentials (UC `Credential` JSON) and
-- the storage URL prefixes they govern.
CREATE TABLE sharing_credentials (
    name    TEXT PRIMARY KEY,
    sealed  BYTEA NOT NULL
);

CREATE TABLE sharing_external_locations (
    name             TEXT PRIMARY KEY,
    url              TEXT NOT NULL,
    credential_name  TEXT NOT NULL REFERENCES sharing_credentials (name)
);
