# olai-uc-files-proto

Generated Rust code for the `portal.files.v1.FilesService` contract — the buffa
message types (with zero-copy views + serde impls) and the `connect-rust` service
trait, dispatcher, and client.

This crate is **generated, not hand-written**. It is the single source of truth
shared by every consumer of the Files API:

- [`olai-uc-files-api`](../files-api) — the native standalone / embeddable Files
  API server implements the `connect::portal::files::v1::FilesService` trait.
- [`olai-uc-query-wasm`](../../wasm/query-wasm) — the in-browser engine implements
  the same trait for the volume-files write path; it depends on this crate
  cross-workspace via `path = "../../crates/files-proto"`.

## Why a shared crate

The generated code names only `buffa`, `buffa-types`, `connectrpc`, and `serde` —
no `object_store`, arrow, or wasm-specific type — so it compiles identically in
the native root workspace and under the wasm workspace's arrow/`object_store`
`[patch.crates-io]` fork. One crate therefore backs both consumers despite their
independent `Cargo.lock`s.

## Regenerating

```bash
just generate-files-proto
```

Never hand-edit anything under `src/generated/`. Change the proto
(`proto/portal/portal/files/v1/svc.proto`) or `buf.gen.files.yaml` and regenerate,
committing the output in the same commit as the source change.
