# olai-uc-credentials

Storage credential vending shared by the Unity Catalog server and the Open
Sharing server.

Given a registered storage credential (AWS IAM role, Azure service principal,
managed identity or storage key) and a storage URL, this crate mints a
short-lived credential downscoped to that URL and operation: an STS
`AssumeRole` session with an inline policy, or an Azure user-delegation /
service SAS. Results are cached until shortly before expiry.

- `location`: storage URL parsing and longest-prefix location matching.
- `vending`: minting and caching temporary credentials.
- `resolve`: the `CredentialResolver` seam a host implements to map a location
  onto its governing credential.

Consuming a vended credential (building an object store, presigning URLs) lives
in [`olai-uc-object-store`](https://crates.io/crates/olai-uc-object-store).
