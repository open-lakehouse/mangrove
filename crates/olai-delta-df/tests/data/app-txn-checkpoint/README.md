# `app-txn-checkpoint` test fixture

A small classic-checkpointed Delta table (checkpoint at version 1, two `modified=`
partitions), vendored from
[`delta-kernel-rs`](https://github.com/delta-io/delta-kernel-rs) at
`kernel/tests/data/app-txn-checkpoint` (Apache-2.0, same license as this repo).

Used by `tests/m2_snapshot_pm_native.rs` to exercise the async checkpoint-footer
`SchemaQuery` read on the P&M snapshot-construction path and the classic-checkpoint
scan path. It is committed here (rather than referenced from a sibling checkout) so
the tests run in CI without an external repo on disk.
