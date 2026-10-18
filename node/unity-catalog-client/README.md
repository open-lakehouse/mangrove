# `@open-lakehouse/unity-catalog-client`

The **data-fetching half** of the Unity Catalog UI: the typed client, the
React-Query hooks, and the generated UC OpenAPI types. Split out from the
presentational `@open-lakehouse/unity-catalog-ui` package so the two concerns —
*talking to Unity Catalog* and *rendering it* — are not bundled together.

## Why it's separate

The presentational package should not hard-wire *how* UC data is fetched. This
package owns that, behind a seam, so the transport can be swapped without
touching a single component.

**UC coverage is fixed.** The hooks here encode the Unity Catalog
entity/resource/securable model (catalogs, schemas, tables, volumes, functions,
models, credentials, external locations) that the UI components are written
against — so the client's *API* is not the pluggable part. What's pluggable is
the transport **underneath** it (the same shape as headwaters' swappable
ConnectRPC `Transport`, one layer down at the `fetch`).

## The seam

The default client issues `openapi-fetch` requests over the platform `fetch`. A
host redirects every request without rebuilding anything:

```ts
import { setDefaultUnityCatalogFetch } from "@open-lakehouse/unity-catalog-client";

// Route UC calls through a host-supplied fetch (auth, an IPC bridge, or an
// in-process wasm UC client that answers /api/2.1/unity-catalog/* directly).
setDefaultUnityCatalogFetch(hostFetch);
```

or build a client with a fixed transport and inject it:

```tsx
<UnityCatalogProvider client={createUnityCatalogClient({ fetch: hostFetch })}>
```

A future **wasm UC client** is exactly this: a `fetch`-shaped adapter that
serves UC requests in-process. It requires **no change to the hooks or the
components** — that is the point of keeping the client separate and swapping only
the transport.

> Follow-up (not built yet): if a raw-`fetch` adapter proves awkward for the wasm
> client (mimicking `Request`/`Response`/URL parsing), a slightly higher-level UC
> transport interface could sit between the client and `fetch`. Deferred until
> the wasm client work starts.

## Public surface

Two entry points:

**`@open-lakehouse/unity-catalog-client`** (framework-free; any runtime with a
`fetch`, no React needed):

- **Client:** `createUnityCatalogFetchClient`, `fetchClient` (the default
  instance), `setDefaultUnityCatalogFetch`, `UnityCatalogFetchClient`,
  `CreateUnityCatalogClientOptions`.
- **Helpers:** `parseUcError`, `objectFullName`.
- **UC OpenAPI types:** `CatalogInfo`, `SchemaInfo`, `TableInfo`, `Create*`,
  `List*`, `components`, `paths`, … (re-exported from `uc-types`).

```ts
import { createUnityCatalogFetchClient } from "@open-lakehouse/unity-catalog-client";

const uc = createUnityCatalogFetchClient({ baseUrl: "http://localhost:8080/api/2.1/unity-catalog" });
const { data, error } = await uc.GET("/catalogs");
```

**`@open-lakehouse/unity-catalog-client/react`** (React + TanStack Query; also
re-exports everything above):

- **Client:** `createUnityCatalogClient`, `defaultUnityCatalogClient`,
  `UnityCatalogClient`, `$api`.
- **Provider:** `UnityCatalogProvider`, `useUnityCatalog`.
- **Read hooks / query helpers:** `useCatalogs`, `useSchemas`, `useTables`,
  `useVolumes`, `useFunctions`, `useModels`, `useCredentials`,
  `useExternalLocations`, the `*Detail` hooks, `prefetch*`, `*DetailQuery`,
  `ucListKeys`.
- **Mutations / invalidators:** the `useCreate*` / `useUpdate*` / `useDelete*`
  hooks and the `invalidate*` / `remove*` helpers.

`react`, `react-dom` and `@tanstack/react-query` are **optional** peers: only the
`./react` entry needs them.

## Codegen

- `bun run gen:api` — regenerate `src/uc-api.d.ts` from the canonical
  proto-derived spec `../../openapi/openapi.yaml` (produced by
  `just generate-openapi`) via `openapi-typescript`.

> These types are generated from mangrove's own canonical spec, so they stay in
> sync with the running server. Regeneration is wired into `just generate-openapi`
> and a CI drift guard fails the build if `uc-api.d.ts` is stale — never hand-edit
> it. The three inlined enums (`TableType`, `DataSourceFormat`, `VolumeType`) are
> derived from their carrying fields in `uc-types.ts` because gnostic inlines enum
> values rather than emitting named component schemas.

## Install

Published to **GitHub Packages**. Point the `@open-lakehouse` scope at it in
`.npmrc`:

```ini
@open-lakehouse:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}
```

```bash
# framework-free client (browsers, Node ≥ 18, Bun, Deno)
bun add @open-lakehouse/unity-catalog-client
# React hosts using ./react also need its peers
bun add @open-lakehouse/unity-catalog-client @tanstack/react-query openapi-react-query react react-dom
```

GitHub Packages needs a token even to read. In GitHub Actions, the job's
`GITHUB_TOKEN` works (`permissions: packages: read`) once the consuming
repository has been granted read access in the package's settings. Elsewhere,
use a token with `read:packages`. Release tarballs carry a build-provenance
attestation (`gh attestation verify <file> -R open-lakehouse/mangrove`).

The package ships ESM (`dist/`, with declarations and source maps). Inside this
workspace, consumers resolve the `source` export condition straight to `src/`
instead (Vite `resolve.conditions` / TS `customConditions`), so no build step is
needed for local development. The `VITE_API_URL` default base URL only applies
when the host is built with Vite; pass `baseUrl` explicitly elsewhere.
