// Rich, deterministic reconciled-Delta-log fixture data — the dataset the
// Delta-log UI is built against until the real wasm runner lands. It mirrors the
// async-native `ReconciledLogProvider` scan-file-row schema
// (crates/olai-delta-df/src/log_explorer.rs): the flat `sm_plans` shape with
// top-level `path` / `size` plus the nested `deletionVector`,
// `fileConstantValues` (incl. `partitionValues_parsed`) and `stats` structs.
//
// Everything is derived from the row index (no Math.random) so stories and tests
// are stable, but with enough variation — nulls, present/absent deletion
// vectors, differing partitions and magnitudes, mixed tight bounds — to look
// real and to exercise the DataGrid's number/string/struct cell formatters.

/** A deletion-vector descriptor, as the kernel surfaces it. Null on most files. */
export interface DeletionVectorFixture {
  storageType: string;
  pathOrInlineDv: string;
  offset: number | null;
  sizeInBytes: number;
  cardinality: number;
}

/** Parsed partition values for a partitioned table (a nested struct). */
export interface PartitionValuesFixture {
  date: string;
  region: string;
}

/** Per-file constant values the kernel attaches to each surviving add. */
export interface FileConstantValuesFixture {
  baseRowId: bigint;
  defaultRowCommitVersion: bigint;
  tags: string | null;
  clusteringProvider: string | null;
  partitionValues_parsed: PartitionValuesFixture;
}

/** Per-file statistics (informational view of `add.stats_parsed`). */
export interface FileStatsFixture {
  numRecords: bigint;
  nullCount: { id: number; amount: number; note: number };
  minValues: { id: number; amount: number; note: string };
  maxValues: { id: number; amount: number; note: string };
  tightBounds: boolean;
}

/** One reconciled-log row — the flat scan-file-row shape. */
export interface ReconciledLogRow {
  path: string;
  size: bigint;
  deletionVector: DeletionVectorFixture | null;
  fileConstantValues: FileConstantValuesFixture;
  stats: FileStatsFixture;
}

const REGIONS = ["us-east", "us-west", "eu-central", "ap-south"];
const CLUSTERING = ["liquid", null, null, "hilbert"];

// A stable pseudo-hex chunk derived from an integer, for UUID-ish file names.
function hex(n: number, width: number): string {
  return (n >>> 0).toString(16).padStart(width, "0").slice(0, width);
}

/**
 * The deterministic reconciled-log dataset. `count` rows, each fully derived
 * from its index. Roughly one in five files carries a deletion vector; nulls,
 * partitions, magnitudes and tight-bounds all vary across the set.
 */
export function reconciledLogFixtureRows(count = 40): ReconciledLogRow[] {
  const rows: ReconciledLogRow[] = [];
  for (let i = 0; i < count; i++) {
    const region = REGIONS[i % REGIONS.length];
    // Partition dates walk backwards a day at a time in groups of four.
    const day = 1 + (Math.floor(i / 4) % 28);
    const date = `2026-05-${String(day).padStart(2, "0")}`;

    const numRecords = BigInt(1_000 + i * 137);
    const size = BigInt(48_000 + i * 9_137 + (i % 7) * 1_024);
    const hasDv = i % 5 === 2; // present on ~1/5 of files, absent (null) otherwise

    const idMin = i * 1000;
    const idMax = idMin + Number(numRecords) - 1;
    const amountMin = Number((0.5 + (i % 9) * 3.25).toFixed(2));
    const amountMax = Number((amountMin + 100 + i * 1.5).toFixed(2));

    rows.push({
      path: `part-${String(i).padStart(5, "0")}-${hex(i * 2_654_435_761, 8)}-${hex(
        i * 40_503,
        4,
      )}-c000.snappy.parquet`,
      size,
      deletionVector: hasDv
        ? {
            storageType: "u",
            pathOrInlineDv: `deletion_vector_${hex(i * 99_991, 8)}.bin`,
            offset: i % 2 === 0 ? 0 : 1 + (i % 16),
            sizeInBytes: 32 + (i % 11) * 8,
            cardinality: 1 + (i % 23),
          }
        : null,
      fileConstantValues: {
        baseRowId: BigInt(i * 10_000),
        defaultRowCommitVersion: BigInt(Math.floor(i / 4)),
        // Most files have no tags; a few carry a small tag map (as a string).
        tags: i % 6 === 0 ? `{"ingested_by":"job-${i % 3}"}` : null,
        clusteringProvider: CLUSTERING[i % CLUSTERING.length],
        partitionValues_parsed: { date, region },
      },
      stats: {
        numRecords,
        nullCount: { id: 0, amount: i % 4, note: (i * 3) % 17 },
        minValues: {
          id: idMin,
          amount: amountMin,
          note: `note-${hex(i, 3)}`,
        },
        maxValues: {
          id: idMax,
          amount: amountMax,
          note: `note-${hex(i + count, 3)}`,
        },
        tightBounds: i % 3 !== 0,
      },
    });
  }
  return rows;
}
