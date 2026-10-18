# Design contract — `@open-lakehouse/files-connect`

Typed ConnectRPC bindings for `portal.files.v1.FilesService`, the write-capable
files API that mangrove's `olai-uc-files-api` (and the in-browser wasm build of
it) serves. It provides the generated messages, a client factory, and a
transport registry. Framework-free: no React, no styles.

## Install & peers

Dependencies: `@bufbuild/protobuf` 2, `@connectrpc/connect` 2. No peers.

## Seams

| Swap point | Contract |
| --- | --- |
| `registerFilesTransport(transport)` | any Connect `Transport`: `@connectrpc/connect-web` over HTTP to a files-api server, or `createWasmFilesTransport(backend)` |
| `createWasmFilesTransport(backend)` | adapts an in-process `FilesBackend` (one unary binary-proto dispatch per RPC) into a `Transport`, with no network hop |

Until a transport is registered, `hasFilesTransport()` is `false` and calls throw
`NoFilesTransportError`.

## Usage

```ts
import { createConnectTransport } from "@connectrpc/connect-web";
import { createFilesClient, registerFilesTransport } from "@open-lakehouse/files-connect";

registerFilesTransport(createConnectTransport({ baseUrl: "https://uc.example.com/files" }));

const files = createFilesClient(); // uses the registered transport
const meta = await files.getFileMetadata({ path: "/Volumes/main/default/raw/a.csv" });
```

## Rules

- **Do** pick the transport once at startup; components and runners call
  `createFilesClient()` and stay transport-agnostic.
- **Don't** hand-edit `src/gen/**`; regenerate from the `portal.files.v1` proto
  with `just generate-files-es`.
