# Design contract — `@open-lakehouse/files`

The volume-files **seam**: list, stat and read files in a Unity Catalog volume
through a host-registered `FilesRunner`, plus React hooks over it. It ships no
transport and no UI. A **leaf**: it never imports Unity Catalog packages (the
volume-browser UI that uses it lives in `@open-lakehouse/unity-catalog-ui`).

## Install & peers

No runtime dependencies. Peers: `react` / `react-dom` 19. No styles.

## Seams

| Layer | Swap point | Contract |
| --- | --- | --- |
| Backend | `registerFilesRunner(runner)` | `FilesRunner`: `listDirectory` (one bounded page), `readFile` (`AsyncIterable<FileChunk>`), `stat`; optional `writeFile` |
| Service | `<FilesServiceProvider service={…}>` / `setDefaultFilesService` | `FilesService` built on the runner (`createFilesService`) |

`hasFilesRunner()` gates the files UI; `filesRunnerCanWrite()` gates upload/edit
affordances, because `writeFile` is optional. Every runner method receives an
`AbortSignal`. Implementations: `@open-lakehouse/files-wasm` (in-browser), or
anything that speaks the portal.files.v1 contract (see
`@open-lakehouse/files-connect`). `@open-lakehouse/files/testing` provides a stub
runner for stories and tests.

## Usage

```tsx
import { FilesServiceProvider, registerFilesRunner, useDirectory } from "@open-lakehouse/files";

registerFilesRunner(myRunner);

function Listing({ path }: { path: string }) {
  const { entries, isLoading, error, hasMore, loadMore } = useDirectory(path, { pageSize: 100 });
  // render entries; call loadMore() for the next page
}

<FilesServiceProvider>
  <Listing path="/Volumes/main/default/raw" />
</FilesServiceProvider>;
```

Path helpers (`parseVolumePath`, `joinVolumePath`, `formatVolumePath`,
`volumeFullName`) convert between `/Volumes/<catalog>/<schema>/<volume>/…` paths
and their parts.

## Rules

- **Do** page directory listings (`listDirectory` returns one page plus a
  `nextPageToken`); never load a whole tree.
- **Do** stream file reads as chunks rather than buffering whole files.
- **Don't** check runner capabilities by probing; use `filesRunnerSupports` /
  `filesRunnerCanWrite`.
