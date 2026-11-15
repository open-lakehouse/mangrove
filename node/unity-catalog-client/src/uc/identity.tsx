// React access to the identity seam (see ../identity.ts).
//
// The directory is, in order: the `identity` of the nearest
// `<IdentityProvider>`, else the `identity` capability of the nearest
// `@open-lakehouse/host` host, else none.
import { useOptionalHost } from "@open-lakehouse/host/react";
import { createContext, type ReactNode, useContext } from "react";
import type { IdentityHost } from "../identity.js";

const IdentityContext = createContext<IdentityHost | null>(null);

/** Provide a principal directory to the UC UI, for embedders without a host
 *  (or to override the host's). */
export function IdentityProvider({
  identity,
  children,
}: {
  identity: IdentityHost;
  children: ReactNode;
}) {
  return (
    <IdentityContext.Provider value={identity}>
      {children}
    </IdentityContext.Provider>
  );
}

/** The principal directory in scope, or `undefined` when there is none. Gate
 *  search and "you" hints on it; without one, take principals as typed. */
export function useIdentityHost(): IdentityHost | undefined {
  const provided = useContext(IdentityContext);
  const host = useOptionalHost();
  return provided ?? host?.identity ?? undefined;
}
