// An `IdentityHost` over the server's read-only SCIM API
// (`/api/2.0/preview/scim/v2`, see openapi/scim.yaml). The server reads users
// from its identity provider and groups and service accounts from its
// authorization store, so a host whose principals live behind the UC server
// can hand this to `<IdentityProvider>` or expose it as its `identity`.
// Framework-free.

import { getHost } from "@open-lakehouse/host";
import { unityCatalogFetchFor } from "./fetch.js";
import type {
  IdentityHost,
  PrincipalKind,
  PrincipalRef,
  PrincipalSearchOptions,
} from "./identity.js";

/** Where the server mounts the SCIM API, from its root. */
export const SCIM_BASE_PATH = "/api/2.0/preview/scim/v2";

const DEFAULT_LIMIT = 20;

/** Options for {@link createScimIdentityHost}. */
export interface ScimIdentityHostOptions {
  /**
   * The fetch to call the server with. Defaults to the Unity Catalog client's
   * default transport (the registered host's `unity-catalog` binding, else the
   * platform `fetch`), so paths are relative to the server root.
   */
  fetch?: typeof globalThis.fetch;
  /** The SCIM API's base URL. Defaults to {@link SCIM_BASE_PATH}. */
  baseUrl?: string;
}

/** A SCIM request the server refused. */
export class ScimRequestError extends Error {
  constructor(
    readonly status: number,
    detail: string,
  ) {
    super(detail);
    this.name = "ScimRequestError";
  }
}

interface ScimUser {
  id: string;
  userName: string;
  displayName?: string;
  active?: boolean;
}

interface ScimNamed {
  id: string;
  displayName: string;
}

const RESOURCES: Record<PrincipalKind, string> = {
  user: "Users",
  group: "Groups",
  service_account: "ServicePrincipals",
};

/** `value` as a SCIM string literal. */
function literal(value: string): string {
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

function filterFor(kind: PrincipalKind, query: string): string {
  const q = literal(query);
  return kind === "user"
    ? `userName co ${q} or displayName co ${q}`
    : `displayName co ${q}`;
}

function toRef(
  kind: PrincipalKind,
  resource: ScimUser | ScimNamed,
): PrincipalRef {
  if (kind === "user") {
    const user = resource as ScimUser;
    return user.displayName
      ? { kind, name: user.userName, displayName: user.displayName }
      : { kind, name: user.userName };
  }
  return { kind, name: (resource as ScimNamed).displayName };
}

/**
 * An {@link IdentityHost} over the server's SCIM API: `search` asks `Users`,
 * `Groups` and `ServicePrincipals` in parallel (users by user name or display
 * name, the others by name) and `current` asks `Me`.
 */
export function createScimIdentityHost(
  options: ScimIdentityHostOptions = {},
): IdentityHost {
  const doFetch = options.fetch ?? unityCatalogFetchFor(getHost);
  const base = (options.baseUrl ?? SCIM_BASE_PATH).replace(/\/$/, "");

  async function get<T>(path: string, signal?: AbortSignal): Promise<T> {
    const response = await doFetch(`${base}${path}`, {
      headers: { accept: "application/scim+json, application/json" },
      signal,
    });
    if (!response.ok) {
      let detail = `SCIM request failed (${response.status})`;
      try {
        const body = (await response.json()) as { detail?: string };
        if (body.detail) detail = body.detail;
      } catch {
        // Not a SCIM error body; keep the status.
      }
      throw new ScimRequestError(response.status, detail);
    }
    return (await response.json()) as T;
  }

  return {
    async search(query: string, opts: PrincipalSearchOptions = {}) {
      const trimmed = query.trim();
      if (!trimmed) return [];
      const limit = opts.limit ?? DEFAULT_LIMIT;
      const kinds =
        opts.kinds ?? (["user", "group", "service_account"] as const);
      const pages = await Promise.all(
        kinds.map(async (kind) => {
          const params = new URLSearchParams({
            filter: filterFor(kind, trimmed),
            count: String(limit),
          });
          const page = await get<{ Resources?: (ScimUser | ScimNamed)[] }>(
            `/${RESOURCES[kind]}?${params}`,
            opts.signal,
          );
          return (page.Resources ?? []).map((r) => toRef(kind, r));
        }),
      );
      return pages.flat().slice(0, limit);
    },

    async current() {
      try {
        const me = await get<ScimUser & { schemas?: string[] }>("/Me");
        const isServicePrincipal = me.schemas?.some((s) =>
          s.endsWith(":ServicePrincipal"),
        );
        return isServicePrincipal
          ? toRef("service_account", me as unknown as ScimNamed)
          : toRef("user", me);
      } catch (error) {
        // Not signed in, or a server without the SCIM API.
        if (
          error instanceof ScimRequestError &&
          (error.status === 401 || error.status === 404)
        ) {
          return null;
        }
        throw error;
      }
    },
  };
}
