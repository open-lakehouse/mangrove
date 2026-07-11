# olai-uc-object-store

[`object_store`](https://docs.rs/object_store) integration for
[Unity Catalog](https://www.unitycatalog.io/). Vends short-lived,
least-privilege credentials through UC's credential APIs and wraps the
result in a regular `Arc<dyn ObjectStore>` — anything that already
accepts one (DataFusion, `delta_kernel`, `parquet`, …) can read and
write data governed by UC volumes, tables, and external locations with
no extra plumbing.

> [!NOTE]
> The Rust crate identifier is `unitycatalog_object_store` (imports and
> paths use that name); the crate is published to crates.io as
> `olai-uc-object-store` while the naming settles. This is an experimental
> component of an unofficial
> [Unity Catalog](https://www.unitycatalog.io/) implementation.

## Install

The `package` rename keeps the `unitycatalog_object_store` import path:

```toml
[dependencies]
unitycatalog-object-store = { package = "olai-uc-object-store", version = "0.0.1" }
object_store = "0.12"
```

## The `uc://` URL scheme

| URL shape                                                  | Vending endpoint                |
|------------------------------------------------------------|---------------------------------|
| `uc:///Volumes/<catalog>/<schema>/<volume>[/<path>]`       | `temporary-volume-credentials`  |
| `uc:///Tables/<catalog>/<schema>/<table>`                  | `temporary-table-credentials`   |
| `s3://`, `gs://`, `abfss://`, `r2://`, `az://`, …          | `temporary-path-credentials`    |

The Databricks-flavoured `vol+dbfs:/Volumes/<c>/<s>/<v>[/<path>]` alias is
also accepted for compatibility with existing credential-vending samples.

## Rust quickstart

```rust
use unitycatalog_object_store::{Operation, UnityObjectStoreFactory};

let factory = UnityObjectStoreFactory::builder()
    .with_uri("https://<workspace>/api/2.1/unity-catalog/")
    .with_token(std::env::var("DATABRICKS_TOKEN").ok())
    .build()
    .await?;

// Address any UC securable with a single URL.
let store = factory
    .for_url("uc:///Volumes/main/default/landing/raw/", Operation::Read)
    .await?;

// `store.as_dyn()` is rooted at the volume's storage location; pass
// paths inside the volume.
let object = store.as_dyn().get(&"file.parquet".into()).await?;
```

### DataFusion

```rust
use datafusion::execution::runtime_env::RuntimeEnv;

let store = factory.for_table("main.default.orders", Operation::Read.into()).await?;
let runtime = RuntimeEnv::default();
runtime
    .object_store_registry
    .register_store(store.url(), store.as_dyn());
```

## Python quickstart

The companion Python integration is shipped as an optional extra on the
`unitycatalog-client` package and returns native
[`obstore`](https://developmentseed.org/obstore/) stores:

```bash
pip install "unitycatalog-client[obstore]"
```

```python
from unitycatalog_client import TemporaryCredentialClient
from unitycatalog_client.obstore import store_for_url
import obstore as obs

creds = TemporaryCredentialClient(base_url=..., token=...)
store = store_for_url(creds, "uc:///Volumes/main/default/landing/raw/")

for entry in obs.list(store).collect():
    print(entry["path"])
```

## Credential lifecycle

The returned `UCStore` carries a credential provider bound to the
originating `(securable, operation)` pair. When the cached credential
approaches expiry, the provider transparently re-vends a fresh one
using the same arguments — so refreshes can never silently widen
privileges across renewals.

In Python, `obstore` itself drives the refresh window from the
`expires_at` field returned by the provider; just call the returned
store and you'll always get a valid credential.

## API surface (Rust)

| Method                                | Returns                                        |
|---------------------------------------|------------------------------------------------|
| `UnityObjectStoreFactory::for_url`    | `UCStore` (parses the URL and dispatches)      |
| `UnityObjectStoreFactory::for_volume` | `UCStore` (volume credentials)                 |
| `UnityObjectStoreFactory::for_table`  | `UCStore` (table credentials)                  |
| `UnityObjectStoreFactory::for_path`   | `UCStore` (path credentials)                   |
| `UnityObjectStoreFactory::dry_run_path` | `UCStore` (`dry_run=true` for permission probes) |
| `UCStore::as_dyn`                     | `Arc<dyn ObjectStore>` (**prefixed** by default) |
| `UCStore::root`                       | `Arc<dyn ObjectStore>` (bucket-rooted escape hatch) |
| `UCStore::url`                        | The cloud URL of the scoped root               |
| `UCStore::prefix`                     | The path within the bucket the credential scopes to |

## License

Apache-2.0
