// Host-contract helpers for UIs that also run without a host.
//
// `@open-lakehouse/host/react`'s `useEnvironmentKey()` throws when there is no
// host, which is right for a host-only UI but not for components that also
// render in the standalone app, in tests or in stories without a host.
import { useOptionalHost } from "@open-lakehouse/host/react";
import { useCallback, useSyncExternalStore } from "react";

const noop = () => {};

/**
 * The current environment's key from the nearest host (`<HostProvider>`, else
 * `registerHost`), or `null` without one. Rerenders when the environment
 * switches. Key a subtree on it so the subtree remounts on a switch.
 */
export function useOptionalEnvironmentKey(): string | null {
  const environment = useOptionalHost()?.environment ?? null;
  // Stable per scope, so React doesn't resubscribe on every render; called as
  // a method in case the scope's `subscribe` uses `this`.
  const subscribe = useCallback(
    (onChange: () => void) =>
      environment ? environment.subscribe(onChange) : noop,
    [environment],
  );
  const key = () => (environment ? environment.key : null);
  return useSyncExternalStore(subscribe, key, key);
}
