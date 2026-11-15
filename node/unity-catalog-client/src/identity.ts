// The identity seam: how a UI finds the principals it can grant to, and who
// is signed in. Framework-free types only.
//
// Identity belongs to the environment, not to Unity Catalog: users live in the
// IdP (Neon Auth, Authelia, …), groups and service accounts in the
// environment's authorization store. So the UI never talks to an IdP itself.
// The host provides an `IdentityHost` (the desktop shell through its own
// backend, a web host over HTTP), as the optional `identity` capability of its
// `@open-lakehouse/host` `LakehouseHost`, or an embedder passes one to
// `<IdentityProvider>`. Without one, principals are typed in by name and the
// server validates them.
//
// These types mirror the `identity` capability planned for
// `@open-lakehouse/host`; they are structural, so a host's implementation
// satisfies both.

/** What a principal is. */
export type PrincipalKind = "user" | "group" | "service_account";

/** A principal a privilege can be granted to. */
export interface PrincipalRef {
  kind: PrincipalKind;
  /**
   * The name the grants API knows it by: a user's email address, a group's
   * name, or a service account's name.
   */
  name: string;
  /** A friendlier label, where the directory has one (a user's full name). */
  displayName?: string;
}

/** Options for {@link IdentityHost.search}. */
export interface PrincipalSearchOptions {
  /** Only these kinds; all kinds when unset. */
  kinds?: PrincipalKind[];
  /** At most this many results. */
  limit?: number;
  /** Aborted when the search is superseded. */
  signal?: AbortSignal;
}

/** The environment's principal directory, as a host provides it. */
export interface IdentityHost {
  /** Principals whose name or display name matches `query`. */
  search(
    query: string,
    options?: PrincipalSearchOptions,
  ): Promise<PrincipalRef[]>;
  /** The signed-in principal, or `null` when the host can't tell. */
  current?(): Promise<PrincipalRef | null>;
}
