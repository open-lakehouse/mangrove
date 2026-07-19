// Seam tests for @open-lakehouse/files, mirroring node/query/src/api.test.ts:
// the registry transitions (hasFilesRunner false -> true), the throwing default
// (reject / throw-on-iteration), capability delegation (supports / canWrite), the
// late-binding stable `filesRunner` ref, and — driving the dev stub — paged
// directory listing continuation via `nextPageToken`.

import { afterEach, beforeEach, expect, test } from "bun:test";
import { createFilesService } from "./api";
import {
  type FilesRunner,
  filesRunner,
  filesRunnerCanWrite,
  filesRunnerSupports,
  hasFilesRunner,
  NoFilesRunnerError,
  registerFilesRunner,
} from "./runner";
import { FIXTURE_VOLUME_ROOT, fixtureListing } from "./testing/fixtures";
import { registerStubFiles, stubFilesRunner } from "./testing/stubRunner";

const never = new AbortController().signal;

// The runner registry is module-level singleton state; reset it to the throwing
// default around each test so ordering can't leak a registration between tests.
// There is no public unregister, so re-install the noop indirectly by registering
// a fresh throwing runner is not equivalent (hasFilesRunner would read true).
// Instead we snapshot nothing and rely on each test registering what it needs;
// the `hasFilesRunner` test runs first against the pristine module.
beforeEach(() => {});
afterEach(() => {});

test("hasFilesRunner() is false before any registration", () => {
  // This must observe the pristine module state, so it asserts before any other
  // test registers a runner. Bun runs tests in file order top-to-bottom.
  expect(hasFilesRunner()).toBe(false);
});

test("the default runner rejects listDirectory/stat with NoFilesRunnerError", async () => {
  // Still pristine at this point (no registration yet).
  await expect(
    filesRunner.listDirectory({ path: FIXTURE_VOLUME_ROOT }, { signal: never }),
  ).rejects.toBeInstanceOf(NoFilesRunnerError);
  await expect(
    filesRunner.stat(FIXTURE_VOLUME_ROOT, { signal: never }),
  ).rejects.toBeInstanceOf(NoFilesRunnerError);
});

test("the default runner's readFile throws NoFilesRunnerError on iteration", async () => {
  const stream = filesRunner.readFile(
    { path: `${FIXTURE_VOLUME_ROOT}/x` },
    { signal: never },
  );
  const iterate = async () => {
    for await (const _ of stream) {
      // unreachable
    }
  };
  await expect(iterate()).rejects.toBeInstanceOf(NoFilesRunnerError);
});

test("hasFilesRunner() flips to true after registerFilesRunner", () => {
  expect(hasFilesRunner()).toBe(false);
  registerStubFiles();
  expect(hasFilesRunner()).toBe(true);
});

test("filesRunnerSupports/filesRunnerCanWrite delegate to declared caps", () => {
  registerFilesRunner(stubFilesRunner, {
    supports: (x) => x.storageScheme === "abfss",
    canWrite: true,
  });
  expect(filesRunnerSupports({ storageScheme: "abfss" })).toBe(true);
  expect(filesRunnerSupports({ storageScheme: "s3" })).toBe(false);
  expect(filesRunnerCanWrite()).toBe(true);

  // Undeclared caps: supports is permissive, canWrite is false.
  registerFilesRunner(stubFilesRunner);
  expect(filesRunnerSupports({ storageScheme: "s3" })).toBe(true);
  expect(filesRunnerCanWrite()).toBe(false);
});

test("late-binding filesRunner resolves the currently-registered runner", async () => {
  let calls = 0;
  const counting: FilesRunner = {
    async listDirectory() {
      calls += 1;
      return { entries: [] };
    },
    readFile: stubFilesRunner.readFile,
    stat: stubFilesRunner.stat,
  };
  registerFilesRunner(counting);
  await filesRunner.listDirectory(
    { path: "/Volumes/a/b/c" },
    { signal: never },
  );
  expect(calls).toBe(1);

  // Re-register a different runner; the SAME stable `filesRunner` ref now routes
  // to it — no re-import, no ordering constraint.
  registerFilesRunner(stubFilesRunner);
  const page = await filesRunner.listDirectory(
    { path: FIXTURE_VOLUME_ROOT, maxResults: 5 },
    { signal: never },
  );
  expect(page.entries.length).toBe(5);
});

test("FilesService.readFile drains the stub stream into one buffer", async () => {
  registerStubFiles();
  const svc = createFilesService();
  const bytes = await svc.readFile({
    path: `${FIXTURE_VOLUME_ROOT}/part-00000.snappy.parquet`,
  });
  // The stub yields two chunks (8 + 4 bytes); readFile concatenates them.
  expect(bytes).toBeInstanceOf(Uint8Array);
  expect(bytes.length).toBe(12);
});

test("FilesService paged listing walks nextPageToken to completion", async () => {
  registerStubFiles();
  const svc = createFilesService();
  const total = fixtureListing(FIXTURE_VOLUME_ROOT).length;
  expect(total).toBeGreaterThan(5); // enough for >=2 pages at pageSize 5

  const collected: string[] = [];
  let token: string | undefined;
  let pages = 0;
  do {
    const page = await svc.listDirectory({
      path: FIXTURE_VOLUME_ROOT,
      maxResults: 5,
      pageToken: token,
    });
    for (const e of page.entries) collected.push(e.path);
    token = page.nextPageToken;
    pages += 1;
  } while (token);

  expect(pages).toBeGreaterThanOrEqual(2);
  expect(collected.length).toBe(total);
  // No duplicates across page boundaries.
  expect(new Set(collected).size).toBe(total);
});

test("FilesService.stat returns fixture metadata; supports/canWrite delegate", async () => {
  registerStubFiles();
  const svc = createFilesService();
  const meta = await svc.stat(
    `${FIXTURE_VOLUME_ROOT}/part-00000.snappy.parquet`,
  );
  expect(meta.path).toBe(`${FIXTURE_VOLUME_ROOT}/part-00000.snappy.parquet`);
  expect(meta.fileSize).toBeGreaterThan(0);
  expect(meta.contentType).toBe("application/vnd.apache.parquet");
  expect(svc.supports({ storageScheme: "gs" })).toBe(true);
  expect(svc.canWrite()).toBe(false);
});
