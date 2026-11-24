# Rotating the server's KEK

The Unity Catalog server seals its secrets at rest under data keys (DEKs) that a
key-encryption key (KEK) wraps. This runbook rotates that KEK: a new version
becomes active, a sweep moves every sealed value onto it, the counts confirm
nothing is left behind, and the old version is retired.

Values are sealed in three places. All three are counted and swept the same way:

| Store | What is sealed | Server |
|---|---|---|
| `objects` | sensitive fields of stored objects | swept by the server |
| `storage_credentials` | storage credentials' secrets (`uc_credentials`) | swept by the server |
| `recipient_tokens` | Delta Sharing recipient tokens awaiting activation | not held by this server (`not_configured`); the host that keeps a sharing store sweeps it with `olai-uc-sharing`'s `SealedRecipientTokens` |

Reads never re-wrap a value. Only the sweep moves values to the active KEK, so a
rotation isn't done until the sweep has run and the counts are zero.

The examples use `curl` against the admin API. `$UC` is the server's base URL
and `$AUTH` an `Authorization` header for a caller the policy allows
`GetKekUsage` and `RewrapSecrets` on the metastore.

## With the key service

The server is configured with `encryption.provider.key_service` (see the
README's *Encryption at rest*). The keyring lives in trestle; the server never
sees a KEK.

### 0. Import an existing local KEK (once)

A server that ran with a local `active` KEK (`UC_SERVER_KEK`, id `kek-1`) keeps
its values when it moves to the key service. Import the KEK as version 1,
keeping its id as an alias:

```bash
trestle key import <env> unity-catalog-rs/server-kek --from UC_SERVER_KEK --alias kek-1
```

Then switch the server's config from `active` to `provider.key_service` and
restart it. Values sealed under `kek-1` open through the alias. New values are
sealed under `server-kek@v1`. `kek-usage` reports the old values under
`kek-1` until a sweep moves them.

### 1. Rotate

```bash
trestle key rotate <env> unity-catalog-rs/server-kek
```

This adds a version and makes it active, then recreates the secret server. The
UC server keeps running. Its client follows the key and seals new values under
the new version within moments; `active_kek_id` in `kek-usage` shows it:

```bash
curl -s -H "$AUTH" "$UC/api/admin/v1/encryption/kek-usage" | jq .active_kek_id
# "server-kek@v2"
```

While the secret server is recreated (about a second), reads keep working for
values whose DEK is in the server's DEK cache (on by default). New writes wait
for the service.

### 2. Sweep

```bash
curl -s -X POST -H "$AUTH" "$UC/api/admin/v1/encryption/rewrap" | jq
```

Per store, the answer has `before` (usage before the sweep) and a `report`:

- `rewrapped`: values moved to the active version;
- `already_active`: values that were on it already;
- `superseded`: values another writer replaced while the sweep ran. The
  writer's value is kept, and it is sealed under the active version;
- `failed`: values left behind, each with its `id`, `kek_id`, `kind`
  (`malformed`, `key_provider`, `store`) and `message`;
- `kek_usage`: usage after the sweep.

A store that fails as a whole (its listing failed, say) carries an `error`, and
the other stores still run. The sweep is idempotent, so for any failure, fix the
cause and run it again. Values already moved count as `already_active`.

### 3. Check the counts

```bash
curl -s -H "$AUTH" "$UC/api/admin/v1/encryption/kek-usage" \
  | jq '.stores[] | {store, by_kek: .kek_usage.by_kek, unreadable: .kek_usage.unreadable}'
```

Go on only when, in every store, `by_kek` holds nothing but the active version,
and `unreadable` is 0. An `unreadable` value has a header the server can't
parse. Find it by id in a sweep's `failed` list (`kind: malformed`).

Check `recipient_tokens` too, wherever a sharing store keeps them (for
example, distributary's).

### 4. Retire

```bash
trestle key retire <env> unity-catalog-rs/server-kek --below 2 --confirm-swept
```

This raises `min_decrypt`. The key service then refuses to unwrap anything
below it, and the server drops cached DEKs below it as soon as its watch sees
the change. The material stays in the keyring (keyrings are add-only), so a
value the sweep missed can still be recovered by lowering `min_decrypt` again.

## With local KEKs

For a server configured with `encryption.active` / `retired` (the Neon Function
placement, or no trestle):

1. **Add a version and make it active.** Generate 32 random bytes,
   base64-encoded. Make it `active` and move the old one to `retired`:

   ```yaml
   encryption:
     active:  { id: v2, key: { env: UC_SERVER_KEK_V2 } }
     retired:
       - { id: v1, key: { env: UC_SERVER_KEK } }
   ```

   Restart the server. New values are sealed under `v2`, and `v1` values keep
   opening.
2. **Sweep**: `POST /api/admin/v1/encryption/rewrap`, as above.
3. **Check the counts**: `GET /api/admin/v1/encryption/kek-usage` until `v1` and
   `unreadable` are 0 in every store.
4. **Retire.** Remove `v1` from `retired` and restart. Anything still sealed
   under it no longer opens, which step 3 rules out. Keep the old material
   somewhere safe until you are sure.
