# olai-uc-sharing

**The state of a Delta Sharing provider: shares, recipients, their bearer
tokens and the grants between them, with one-time activation codes and key
rotation.**

> [!NOTE]
> The Rust crate identifier is `unitycatalog_sharing` (imports and paths use
> that name); the crate is published to crates.io as `olai-uc-sharing` while the
> naming settles. This is an experimental component of an unofficial
> [Unity Catalog](https://unitycatalog.io) implementation.

The store is split by resource:

- `ShareStore` holds shares (lists of `ShareObject`s naming catalog assets) and
  grants. Its read-only view, `ShareReader`, is all a sharing protocol needs to
  list and authorize shares.
- `RecipientStore` holds recipients and their tokens. Its read-only view,
  `TokenReader`, is all bearer authentication needs. Creating a recipient with
  its first token, rotating a token and redeeming an activation code are each
  one atomic operation.
- `SharingStore` bundles both.

A host that keeps shares and recipients in a store of its own implements only
the read-only views.

`recipients` holds the token rules: issuing a token sealed for activation,
rotating with a grace period, redeeming an activation code exactly once, and
`rewrap_tokens` to re-wrap sealed tokens under the active key after a KEK
rotation. Tokens and activation codes are stored only as sha256 digests
(`tokens::token_hash`).

Implementations:

- `InMemorySharingStore` (feature `memory`), for tests and ephemeral servers;
- `PgSharingStore` (feature `postgres`) and `SqliteSharingStore` (feature
  `sqlite`), over a pool the host provides, each with tables of its own. Their
  `MIGRATOR` (versioned 0301) is meant to be merged into the host's migrator so
  the database keeps one `_sqlx_migrations` ledger.

The `testing` feature exposes the conformance suite every backend runs
(`testing::round_trips`).
