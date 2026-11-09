// The single seam between the Unity Catalog package and the host's environment
// state.
//
// `ExpansionContext` namespaces its persisted tree-expansion state per active
// environment, so it needs the current environment id. That is the ONE inbound
// dependency from UC core that is not itself UC — every other dependency is a
// shared primitive (@open-lakehouse/ui-kit) or the generic client factory
// (./api).
//
// The scope is, in order: the `scopeId` of the nearest
// `<EnvironmentScopeProvider>`, else the environment key of the nearest
// `@open-lakehouse/host` host (`<HostProvider>`, else `registerHost`), else a
// stable constant. So a host that implements the contract needs no wiring, an
// embedder without one can still pass its own id (the standalone app passes
// "uc"), and tests and stories without either share one namespace. See
// ../README.md.
import { useOptionalEnvironmentKey } from "@open-lakehouse/unity-catalog-client/react";
import {
  createContext,
  createElement,
  type ReactNode,
  useContext,
} from "react";

/** Namespace used when neither a scope nor a host is provided. */
const DEFAULT_SCOPE_ID = "default";

const EnvironmentScopeContext = createContext<string | null>(null);

/**
 * Provide the id that namespaces per-environment UC UI state (currently tree
 * expansion). Without `scopeId`, the scope is the host's environment key, so it
 * changes on every environment switch.
 */
export function EnvironmentScopeProvider({
  scopeId,
  children,
}: {
  scopeId?: string;
  children: ReactNode;
}) {
  const hostKey = useOptionalEnvironmentKey();
  return createElement(
    EnvironmentScopeContext.Provider,
    { value: scopeId ?? hostKey ?? DEFAULT_SCOPE_ID },
    children,
  );
}

/**
 * The id used to namespace per-environment UI state (currently tree
 * expansion): the nearest {@link EnvironmentScopeProvider}'s scope, else the
 * host's environment key, else a stable constant.
 */
export function useEnvironmentScopeId(): string {
  const provided = useContext(EnvironmentScopeContext);
  const hostKey = useOptionalEnvironmentKey();
  return provided ?? hostKey ?? DEFAULT_SCOPE_ID;
}
