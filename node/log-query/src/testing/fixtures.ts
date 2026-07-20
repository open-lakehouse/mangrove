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

/** Per-file statistics (informational view of `add.stats_parsed`). The
 *  `minValues`/`maxValues` sub-structs are omitted for a table with no
 *  skipping-eligible columns — the min/max-box empty-axes case. */
export interface FileStatsFixture {
  numRecords: bigint;
  nullCount: { id: number; amount: number; note: number };
  minValues?: { id: number; amount: number; note: string };
  maxValues?: { id: number; amount: number; note: string };
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
 *
 * `withStats` controls whether the `minValues`/`maxValues` sub-structs are
 * populated: when `false`, every file's stats carry only `numRecords` /
 * `nullCount` / `tightBounds` (the shape a table with no skipping-eligible
 * columns produces), which drives the min/max-box empty-axes path.
 */
export function reconciledLogFixtureRows(
  count = 40,
  withStats = true,
): ReconciledLogRow[] {
  const rows: ReconciledLogRow[] = [];
  for (let i = 0; i < count; i++) {
    const region = REGIONS[i % REGIONS.length];
    // Partition dates walk backwards a day at a time in groups of four.
    const day = 1 + (Math.floor(i / 4) % 28);
    const date = `2026-05-${String(day).padStart(2, "0")}`;

    const numRecords = BigInt(1_000 + i * 137);
    const size = BigInt(48_000 + i * 9_137 + (i % 7) * 1_024);
    const hasDv = i % 5 === 2; // present on ~1/5 of files, absent (null) otherwise

    // Deliberately overlapping ranges: files advance their `id` window by only
    // ~40% of its width per step, so consecutive files share id space — the
    // interesting min/max-box case (overlap = weaker data skipping). `amount`
    // ranges are wider and drift more slowly, giving heavy 2D overlap.
    const idWidth = Number(numRecords) - 1;
    const idMin = i * Math.round(idWidth * 0.4);
    const idMax = idMin + idWidth;
    const amountMin = Number((0.5 + (i % 6) * 12.5).toFixed(2));
    const amountMax = Number((amountMin + 140 + (i % 5) * 20).toFixed(2));

    const stats: FileStatsFixture = {
      numRecords,
      nullCount: { id: 0, amount: i % 4, note: (i * 3) % 17 },
      tightBounds: i % 3 !== 0,
    };
    if (withStats) {
      stats.minValues = {
        id: idMin,
        amount: amountMin,
        note: `note-${hex(i, 3)}`,
      };
      stats.maxValues = {
        id: idMax,
        amount: amountMax,
        note: `note-${hex(i + count, 3)}`,
      };
    }

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
      stats,
    });
  }
  return rows;
}

// --- Reconciled action-stream fixture (the `actions` log surface) ------------
//
// Mirrors the async-native `ActionsLogProvider` (crates/olai-delta-df): six
// nullable top-level struct slots — add / remove / metaData / protocol /
// domainMetadata / txn — with EXACTLY ONE non-null per row (the reconciled full
// action stream). This is the shape that makes a flat grid unreadable (5/6 of
// every row is null) and that the rich action-row view renders instead.
//
// `add` carries the parsed stats sub-struct (`stats_parsed`), matching the
// provider (raw JSON `stats` string is replaced by the typed struct).

/** The `add` action slot — a surviving file plus its parsed stats. */
export interface AddActionFixture {
  path: string;
  size: bigint;
  dataChange: boolean;
  stats_parsed: { numRecords: bigint; tightBounds: boolean };
}

/** One reconciled action row: exactly one slot is non-null. */
export interface ActionLogRow {
  add: AddActionFixture | null;
  remove: {
    path: string;
    dataChange: boolean;
    deletionTimestamp: bigint;
  } | null;
  metaData: { id: string; name: string | null; format: string } | null;
  protocol: { minReaderVersion: number; minWriterVersion: number } | null;
  domainMetadata: { domain: string; removed: boolean } | null;
  txn: { appId: string; version: bigint } | null;
}

const EMPTY_SLOTS: ActionLogRow = {
  add: null,
  remove: null,
  metaData: null,
  protocol: null,
  domainMetadata: null,
  txn: null,
};

/**
 * A deterministic reconciled action stream. The first six rows populate one
 * distinct slot each (so `tableFromJSON` settles every struct's schema from a
 * non-null example — the "build the full shape first" requirement), followed by
 * a realistic mix dominated by `add`/`remove` with occasional metadata/txn.
 */
export function actionsLogFixtureRows(count = 40): ActionLogRow[] {
  const rows: ActionLogRow[] = [];

  // Seed rows: one populated slot each, in slot order, to fix all six schemas.
  rows.push({ ...EMPTY_SLOTS, add: addAt(0) });
  rows.push({ ...EMPTY_SLOTS, remove: removeAt(1) });
  rows.push({
    ...EMPTY_SLOTS,
    metaData: { id: "b1e6…-table", name: "events", format: "parquet" },
  });
  rows.push({
    ...EMPTY_SLOTS,
    protocol: { minReaderVersion: 3, minWriterVersion: 7 },
  });
  rows.push({
    ...EMPTY_SLOTS,
    domainMetadata: { domain: "delta.liquid", removed: false },
  });
  rows.push({ ...EMPTY_SLOTS, txn: { appId: "streaming-app", version: 12n } });

  // The remaining rows: mostly adds and removes (as a real log is), with a
  // sprinkle of txn/domainMetadata, each with exactly one slot set.
  for (let i = rows.length; i < count; i++) {
    if (i % 7 === 0) {
      rows.push({
        ...EMPTY_SLOTS,
        txn: { appId: `app-${i % 3}`, version: BigInt(i) },
      });
    } else if (i % 11 === 0) {
      rows.push({
        ...EMPTY_SLOTS,
        domainMetadata: { domain: `delta.tag.${i % 4}`, removed: i % 2 === 0 },
      });
    } else if (i % 3 === 0) {
      rows.push({ ...EMPTY_SLOTS, remove: removeAt(i) });
    } else {
      rows.push({ ...EMPTY_SLOTS, add: addAt(i) });
    }
  }
  return rows;
}

function addAt(i: number): AddActionFixture {
  return {
    path: `part-${String(i).padStart(5, "0")}-${hex(i * 2_654_435_761, 8)}-c000.snappy.parquet`,
    size: BigInt(48_000 + i * 9_137),
    dataChange: true,
    stats_parsed: {
      numRecords: BigInt(1_000 + i * 137),
      tightBounds: i % 3 !== 0,
    },
  };
}

function removeAt(i: number): {
  path: string;
  dataChange: boolean;
  deletionTimestamp: bigint;
} {
  return {
    path: `part-${String(i).padStart(5, "0")}-${hex(i * 40_503, 8)}-c000.snappy.parquet`,
    dataChange: true,
    deletionTimestamp: BigInt(1_714_000_000_000 + i * 60_000),
  };
}
