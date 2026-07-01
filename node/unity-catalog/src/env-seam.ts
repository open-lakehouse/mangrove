// The single seam between the Unity Catalog package and the host's environment
// state.
//
// `ExpansionContext` namespaces its persisted tree-expansion state per active
// environment, so it needs the current environment id. That is the ONE inbound
// dependency from UC core that is not itself UC — every other dependency is a
// shared primitive (@open-lakehouse/ui-kit) or the generic client factory
// (./api).
//
// Rather than reach into the host's context, the package OWNS a small scope
// context here and the host feeds it: mount `<EnvironmentScopeProvider
// scopeId={activeEnvironmentId}>` (see the app's UC wiring). When no provider is
// mounted the scope falls back to a stable constant, so tests and Storybook
// stories render without a wrapper and behave exactly as before. To embed the
// package elsewhere, supply whatever id represents the embedder's "current
// scope" (or nothing, to share one namespace). See ./README.md.
import {
  createContext,
  createElement,
  type ReactNode,
  useContext,
} from "react";

/** Namespace used when no host scope is provided (single shared namespace). */
const DEFAULT_SCOPE_ID = "default";

const EnvironmentScopeContext = createContext<string>(DEFAULT_SCOPE_ID);

/**
 * Provide the id that namespaces per-environment UC UI state (currently tree
 * expansion). The host mounts this with its active-environment id.
 */
export function EnvironmentScopeProvider({
  scopeId,
  children,
}: {
  scopeId: string;
  children: ReactNode;
}) {
  return createElement(
    EnvironmentScopeContext.Provider,
    { value: scopeId },
    children,
  );
}

/**
 * The id used to namespace per-environment UI state (currently tree expansion).
 * Falls back to a stable constant when no {@link EnvironmentScopeProvider} is
 * mounted.
 */
export function useEnvironmentScopeId(): string {
  return useContext(EnvironmentScopeContext);
}
