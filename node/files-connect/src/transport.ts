// A ConnectRPC `Transport` that bridges the `portal.files.v1.FilesService`
// client to an in-browser wasm backend, mirroring hydrofoil's Tauri transport
// (node/desktop/src/tauri-transport.ts) — the same metadata-vs-bytes split.
//
//   - unary METADATA RPCs (GetFileMetadata / ListDirectoryContents / DeleteFile /
//     CreateDirectory / DeleteDirectory / GetDirectoryMetadata):
//     `toBinary(input)` → the backend's `connectUnary(path, bytes)` → the wasm
//     connect dispatcher → `fromBinary(output)`. Binary proto end to end, so the
//     int64 fields decode to proper `bigint`s (no camelCase/number-vs-string JSON
//     ambiguity at the edge).
//   - byte RPCs (UploadFile client-stream / DownloadFile server-stream): bypass
//     the dispatcher entirely — delegate to the backend's native
//     `writeFileBytes` / `readFileBytes` calls with a raw binary body. Streaming
//     file bytes through the connect envelope is what hydrofoil deliberately
//     avoids.
//
// The transport is backend-agnostic: it takes a {@link FilesBackend} of three
// async calls, so the same transport works over a Web Worker, a direct
// main-thread engine, or a test double. `@open-lakehouse/files-wasm` wires the
// worker-backed one.

import {
  create,
  type DescMessage,
  type DescMethodStreaming,
  type DescMethodUnary,
  fromBinary,
  type MessageInitShape,
  type MessageShape,
  toBinary,
} from "@bufbuild/protobuf";
import type { Transport } from "@connectrpc/connect";

/** The three backend calls the wasm engine exposes (see crates/query-wasm's
 *  `UcFilesEngine`): the generic unary proto dispatch, plus the two native byte
 *  transfers. A host wires these to a Web Worker (or calls the engine directly). */
export interface FilesBackend {
  /**
   * Dispatch one unary `portal.files.v1.FilesService` RPC through the wasm
   * connect Router, as binary proto. `path` is the full RPC path
   * (`portal.files.v1.FilesService/GetFileMetadata`); `requestBytes` is the
   * binary-proto request; resolves to the binary-proto response.
   */
  connectUnary(path: string, requestBytes: Uint8Array): Promise<Uint8Array>;
  /**
   * Read a file (or byte range), invoking `onChunk` per body chunk in file order.
   * Bytes bypass the connect dispatcher (native call).
   */
  readFileBytes(
    path: string,
    offset: number | undefined,
    length: number | undefined,
    onChunk: (bytes: Uint8Array) => void,
  ): Promise<void>;
  /**
   * Write (create or overwrite) a file from one buffered body. Bytes bypass the
   * connect dispatcher (native call). Resolves to the post-write metadata.
   */
  writeFileBytes(
    path: string,
    bytes: Uint8Array,
    contentType: string | undefined,
    ifMatchEtag: string | undefined,
  ): Promise<{ path: string; fileSize: number; etag?: string }>;
}

/** The Connect method path, e.g. `portal.files.v1.FilesService/GetFileMetadata`. */
function methodPath(method: {
  parent: { typeName: string };
  name: string;
}): string {
  return `${method.parent.typeName}/${method.name}`;
}

/** A DownloadFile request message (the fields the byte path reads off it). */
interface DownloadRequest {
  path: string;
  offset?: bigint;
  length?: bigint;
}

/** An UploadFile request frame (one message of the client stream). */
interface UploadFrame {
  path: string;
  contentType?: string;
  chunk: Uint8Array;
}

/**
 * Build a ConnectRPC {@link Transport} over a {@link FilesBackend}. The returned
 * transport is handed to `createClient(FilesService, transport)`; unary metadata
 * calls go through binary proto over `backend.connectUnary`, and the two byte
 * RPCs are bridged onto `backend.readFileBytes` / `backend.writeFileBytes`.
 */
export function createWasmFilesTransport(backend: FilesBackend): Transport {
  return {
    async unary<I extends DescMessage, O extends DescMessage>(
      method: DescMethodUnary<I, O>,
      _signal: AbortSignal | undefined,
      _timeoutMs: number | undefined,
      _header: HeadersInit | undefined,
      input: MessageInitShape<I>,
    ) {
      const message = create(method.input, input);
      const requestBytes = toBinary(method.input, message);
      const responseBytes = await backend.connectUnary(
        methodPath(method),
        requestBytes,
      );
      const out = fromBinary(method.output, responseBytes);
      return {
        stream: false as const,
        service: method.parent,
        method,
        header: new Headers(),
        message: out,
        trailer: new Headers(),
      };
    },

    async stream<I extends DescMessage, O extends DescMessage>(
      method: DescMethodStreaming<I, O>,
      _signal: AbortSignal | undefined,
      _timeoutMs: number | undefined,
      _header: HeadersInit | undefined,
      input: AsyncIterable<MessageInitShape<I>>,
    ) {
      const out = method.output;

      if (method.name === "DownloadFile") {
        // Take the single server-stream request, then pull chunks from the
        // native byte reader and re-wrap each as a DownloadFileResponse.
        let req: DownloadRequest | undefined;
        for await (const msg of input) {
          req = create(method.input, msg) as unknown as DownloadRequest;
          break;
        }
        if (!req)
          throw new Error("files transport: DownloadFile missing request");

        const chunks: Uint8Array[] = [];
        await backend.readFileBytes(
          req.path,
          req.offset !== undefined ? Number(req.offset) : undefined,
          req.length !== undefined ? Number(req.length) : undefined,
          (bytes) => chunks.push(bytes),
        );
        async function* messages(): AsyncIterable<MessageShape<O>> {
          for (const chunk of chunks) {
            // The output shape is known at runtime (DownloadFileResponse), not to
            // the generic `O`; funnel through `unknown` as hydrofoil's transport does.
            yield create(out, { chunk } as unknown as MessageInitShape<O>);
          }
        }
        return {
          stream: true as const,
          service: method.parent,
          method,
          header: new Headers(),
          message: messages(),
          trailer: new Headers(),
        };
      }

      if (method.name === "UploadFile") {
        // Drain the request frames (first carries path + contentType, all carry
        // chunk bytes), concatenate, and hand the whole body to the native
        // buffered write. Yields the single UploadFileResponse.
        let path = "";
        let contentType: string | undefined;
        const parts: Uint8Array[] = [];
        let total = 0;
        let first = true;
        for await (const msg of input) {
          const frame = create(method.input, msg) as unknown as UploadFrame;
          if (first) {
            path = frame.path;
            contentType = frame.contentType;
            first = false;
          }
          if (frame.chunk?.length) {
            parts.push(frame.chunk);
            total += frame.chunk.length;
          }
        }
        const body = new Uint8Array(total);
        let offset = 0;
        for (const p of parts) {
          body.set(p, offset);
          offset += p.length;
        }
        const meta = await backend.writeFileBytes(
          path,
          body,
          contentType,
          undefined,
        );
        async function* single(): AsyncIterable<MessageShape<O>> {
          // Runtime-known output shape (UploadFileResponse); funnel through
          // `unknown` — the generic `O` can't see the concrete fields.
          yield create(out, {
            path: meta.path,
            fileSize: BigInt(meta.fileSize),
            etag: meta.etag ?? "",
          } as unknown as MessageInitShape<O>);
        }
        return {
          stream: true as const,
          service: method.parent,
          method,
          header: new Headers(),
          message: single(),
          trailer: new Headers(),
        };
      }

      throw new Error(
        `files transport: unmapped streaming RPC ${method.name} (only Upload/Download bypass the dispatcher)`,
      );
    },
  };
}
