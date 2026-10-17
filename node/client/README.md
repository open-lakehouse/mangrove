# `@open-lakehouse/unity-catalog-node`

Native Unity Catalog client for Node.js: the Rust `olai-uc-client` crate exposed
through [NAPI-RS](https://napi.rs), with a typed TypeScript wrapper and the UC
protobuf models. Requires Node.js ≥ 22.

For browsers (or anywhere you'd rather not load a native addon), use the
fetch-based [`@open-lakehouse/unity-catalog-client`](../unity-catalog-client/)
instead.

## Install

Published to **GitHub Packages**. Point the `@open-lakehouse` scope at it in
`.npmrc`:

```ini
@open-lakehouse:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}
```

```bash
bun add @open-lakehouse/unity-catalog-node   # or npm install / pnpm add
```

GitHub Packages needs a token even to read. In GitHub Actions, the job's
`GITHUB_TOKEN` works (`permissions: packages: read`) once the consuming
repository has been granted read access in the package's settings. Elsewhere,
use a token with `read:packages`.

The native binary ships as a per-platform package
(`@open-lakehouse/unity-catalog-node-<platform>`), installed automatically as an
optional dependency. Prebuilt for Linux (x64/arm64, glibc and musl) and macOS
(x64/arm64).

Release tarballs and binaries carry a build-provenance attestation:

```bash
gh attestation verify <file> -R open-lakehouse/mangrove
```

## Use

```ts
import { UnityCatalogClient } from "@open-lakehouse/unity-catalog-node";

const client = new UnityCatalogClient(
  "http://localhost:8080/api/2.1/unity-catalog/",
  process.env.UC_TOKEN, // optional bearer token
);

await client.createCatalog("demo", { comment: "created from Node" });
const catalog = await client.catalog("demo").get();
const all = await client.listCatalogs();
```

## Development

```bash
bun run build      # debug NAPI binary + TypeScript build into dist/
bun run test       # unit tests (jest)
```

`unitycatalog/codegen/` is generated (TS client + protobuf models, and the
`native.js` / `native.d.ts` loader from `napi build --js --dts`). Regenerate it
rather than editing by hand.
