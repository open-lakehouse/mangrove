// Public surface of `@open-lakehouse/files` — the ONLY entry point other code
// may import from (except the `./testing` subexport for the dev stub).
//
// This package is a LEAF like @open-lakehouse/query and @open-lakehouse/log-query:
// it depends only on React, never on Unity Catalog or any app feature. The Unity
// Catalog package builds on it (renders a volume Files tab), never the reverse —
// enforced by the Biome leaf rule in node/biome.json.
//
// It ships NO runtime files implementation. It provides the seam:
//   - a low-level `FilesRunner` a host / the wasm engine registers, and
//   - a volume-oriented `FilesService` / `useDirectory` / provider on top.
// The dev stub that makes the tab render without wasm lives on the `./testing`
// subexport, not here. See ./runner.ts for why there is no default runner.

// Volume-oriented service surface.
export {
  createFilesService,
  defaultFilesService,
  setDefaultFilesService,
} from "./api";
// React injection + hook.
export { FilesServiceProvider, useFilesService } from "./context";
// Canonical /Volumes/<c>/<s>/<v>/<rest> path helpers.
export {
  formatVolumePath,
  joinVolumePath,
  parseVolumePath,
  type VolumePath,
  volumeFullName,
} from "./path";
// Low-level runner seam (the swap point for the wasm engine / a host).
export {
  type FilesRunner,
  type FilesRunnerCapabilities,
  filesRunner,
  filesRunnerCanWrite,
  filesRunnerSupports,
  getFilesRunner,
  hasFilesRunner,
  NoFilesRunnerError,
  registerFilesRunner,
} from "./runner";
export type {
  DirectoryEntry,
  DirectoryMetadata,
  DirectoryPage,
  FileChunk,
  FileMetadata,
  FilesService,
  FilesSupportsInput,
  ListDirectoryRequest,
  ReadFileRequest,
  WriteFileRequest,
  WriteFileResult,
} from "./types";
export { type DirectoryState, useDirectory } from "./useDirectory";
