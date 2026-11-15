// The identity seam: how a UI finds the principals it can grant to, and who
// is signed in.
//
// Identity belongs to the environment, not to Unity Catalog: users live in the
// IdP (Neon Auth, Authelia, ...), groups and service accounts in the
// environment's authorization store. So the UI never talks to an IdP itself.
// The host provides an `IdentityHost` (the desktop shell through its own
// backend, a web host over HTTP) as the optional `identity` capability of its
// `@open-lakehouse/host` `LakehouseHost`, or an embedder passes one to
// `<IdentityProvider>`. Without one, principals are typed in by name and the
// server validates them.
//
// The types are the host contract's, re-exported so UC code needs one import.

export type {
  IdentityHost,
  PrincipalKind,
  PrincipalRef,
  PrincipalSearchOptions,
} from "@open-lakehouse/host";
