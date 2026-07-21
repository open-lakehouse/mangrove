// The typed `portal.files.v1.FilesService` client, built over the late-binding
// registry transport. `createClient` reads the generated `FilesService`
// descriptor (protoc-gen-es v2 emits it directly — no separate connect-es
// plugin) and dispatches through whatever transport the host registered.

import type { Transport } from "@connectrpc/connect";
import { type Client, createClient } from "@connectrpc/connect";
import { FilesService } from "./gen/portal/files/v1/svc_pb";
import { clientTransport } from "./registry";

/** A typed client for `portal.files.v1.FilesService`. */
export type FilesClient = Client<typeof FilesService>;

/**
 * Build a {@link FilesClient}. With no argument it uses the late-binding registry
 * transport (`clientTransport`), so it works before OR after a host registers the
 * backend; pass an explicit `transport` to bind one directly (e.g. in tests).
 */
export function createFilesClient(
  transport: Transport = clientTransport,
): FilesClient {
  return createClient(FilesService, transport);
}
