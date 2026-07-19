// Editor tab/session reducer — the React-owned, low-frequency state that drives
// the tab strip: which tabs are open, their order, which is active, and each
// tab's save status. The high-frequency hot state (the Monaco model, view state,
// saved-version baseline) lives in the model registry (core/models.ts), NOT here
// — so typing never churns reducer state.

import type { EditorLanguage } from "../core/language";

export type TabId = string; // == file path (unique per store), a stable key.

export type SaveStatus = "clean" | "dirty" | "saving" | "saved" | "error";

export interface OpenTab {
  id: TabId;
  path: string;
  /** Basename, for the tab label. */
  name: string;
  language: EditorLanguage;
  saveStatus: SaveStatus;
  /** Last save error message, when saveStatus === "error". */
  error?: string;
  /** Etag from the last load/save, for write-if-match. */
  etag?: string;
}

export interface SessionState {
  /** Tabs in display order. */
  tabs: OpenTab[];
  /** Active tab id, or null when no tab is open. */
  activeId: TabId | null;
}

export const initialSessionState: SessionState = { tabs: [], activeId: null };

export type SessionAction =
  | {
      type: "OPEN_TAB";
      tab: Omit<OpenTab, "saveStatus">;
    }
  | { type: "ACTIVATE_TAB"; id: TabId }
  | { type: "CLOSE_TAB"; id: TabId }
  | { type: "REORDER_TABS"; from: number; to: number }
  | { type: "SET_STATUS"; id: TabId; saveStatus: SaveStatus; error?: string }
  | { type: "SET_ETAG"; id: TabId; etag: string };

/** The tab to activate after closing `closingId` (its right neighbor, else left). */
function neighborAfterClose(tabs: OpenTab[], closingId: TabId): TabId | null {
  const idx = tabs.findIndex((t) => t.id === closingId);
  if (idx < 0) return null;
  const next = tabs[idx + 1] ?? tabs[idx - 1];
  return next ? next.id : null;
}

export function sessionReducer(
  state: SessionState,
  action: SessionAction,
): SessionState {
  switch (action.type) {
    case "OPEN_TAB": {
      const existing = state.tabs.find((t) => t.id === action.tab.id);
      if (existing) {
        // Already open — just activate it (no duplicate, keep its status).
        return { ...state, activeId: existing.id };
      }
      const tab: OpenTab = { ...action.tab, saveStatus: "clean" };
      return { tabs: [...state.tabs, tab], activeId: tab.id };
    }

    case "ACTIVATE_TAB": {
      if (!state.tabs.some((t) => t.id === action.id)) return state;
      return { ...state, activeId: action.id };
    }

    case "CLOSE_TAB": {
      const tabs = state.tabs.filter((t) => t.id !== action.id);
      const activeId =
        state.activeId === action.id
          ? neighborAfterClose(state.tabs, action.id)
          : state.activeId;
      return { tabs, activeId };
    }

    case "REORDER_TABS": {
      const { from, to } = action;
      if (
        from === to ||
        from < 0 ||
        to < 0 ||
        from >= state.tabs.length ||
        to >= state.tabs.length
      ) {
        return state;
      }
      const tabs = [...state.tabs];
      const [moved] = tabs.splice(from, 1);
      tabs.splice(to, 0, moved);
      return { ...state, tabs };
    }

    case "SET_STATUS": {
      const tabs = state.tabs.map((t) =>
        t.id === action.id
          ? { ...t, saveStatus: action.saveStatus, error: action.error }
          : t,
      );
      return { ...state, tabs };
    }

    case "SET_ETAG": {
      const tabs = state.tabs.map((t) =>
        t.id === action.id ? { ...t, etag: action.etag } : t,
      );
      return { ...state, tabs };
    }

    default:
      return state;
  }
}
