-- Shares, recipients, their bearer tokens and the grants between them.
--
-- Owned by `PgSharingStore`. Secrets are stored sealed (envelope encryption),
-- recipient tokens and activation codes only as sha256 digests.

CREATE TABLE uc_shares (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    comment     TEXT,
    -- The share's objects (UC `DataObject`s), in the order they were added.
    objects     JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at  BIGINT NOT NULL,
    updated_at  BIGINT NOT NULL
);

CREATE TABLE uc_recipients (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    comment     TEXT,
    created_at  BIGINT NOT NULL
);

CREATE TABLE uc_recipient_tokens (
    id                    TEXT PRIMARY KEY,
    recipient_id          TEXT NOT NULL REFERENCES uc_recipients (id) ON DELETE CASCADE,
    token_hash            BYTEA NOT NULL UNIQUE,
    -- Kept only until the recipient activates the token.
    sealed_token          BYTEA,
    -- The sha256 digest of the activation code, which is derived from the
    -- token; the link is rebuilt from the sealed token while that is kept.
    activation_code_hash  BYTEA UNIQUE,
    expires_at            BIGINT,
    created_at            BIGINT NOT NULL
);

CREATE INDEX uc_recipient_tokens_recipient ON uc_recipient_tokens (recipient_id);

CREATE TABLE uc_share_grants (
    share_id      TEXT NOT NULL REFERENCES uc_shares (id) ON DELETE CASCADE,
    recipient_id  TEXT NOT NULL REFERENCES uc_recipients (id) ON DELETE CASCADE,
    PRIMARY KEY (share_id, recipient_id)
);

CREATE INDEX uc_share_grants_recipient ON uc_share_grants (recipient_id);
