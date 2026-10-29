# olai-uc-credentials

Storage credential vending shared by the Unity Catalog server and the Open
Sharing server.

Given a registered storage credential (AWS IAM role, Azure service principal,
managed identity or storage key) and a storage URL, this crate mints a
short-lived credential downscoped to that URL and operation: an STS
`AssumeRole` session with an inline policy, or an Azure user-delegation /
service SAS. Results are cached until shortly before expiry.

- `model`: the crate's own credential types (`StorageCredential` in,
  `TemporaryCredential` out). They depend on no catalog's models and read the
  Unity Catalog REST JSON; `olai-uc-common` (feature `credentials`) converts the
  generated models.
- `location`: storage URL parsing and longest-prefix location matching.
- `vending`: minting and caching temporary credentials.
- `resolve`: the `CredentialResolver` seam a host implements to map a location
  onto its governing credential.

`vending` and `resolve` sit behind the default `vending` feature (native only).
With `default-features = false` the crate is just `model` and `location`, which
build on every target, including `wasm32`.

Consuming a vended credential (building an object store, presigning URLs) lives
in [`olai-uc-object-store`](https://crates.io/crates/olai-uc-object-store).
