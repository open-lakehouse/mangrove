// Unit tests for the tab-session reducer — the pure state machine behind the tab
// strip (open/activate/close/reorder/status/etag). No DOM or Monaco needed.

import { describe, expect, test } from "bun:test";
import {
  initialSessionState,
  type OpenTab,
  type SessionState,
  sessionReducer,
} from "./sessionReducer";

const openTab = (id: string): Omit<OpenTab, "saveStatus"> => ({
  id,
  path: id,
  name: id.split("/").pop() ?? id,
  language: "sql",
});

const withTabs = (...ids: string[]): SessionState =>
  ids.reduce(
    (s, id) => sessionReducer(s, { type: "OPEN_TAB", tab: openTab(id) }),
    initialSessionState,
  );

describe("OPEN_TAB", () => {
  test("opens and activates a new tab", () => {
    const s = sessionReducer(initialSessionState, {
      type: "OPEN_TAB",
      tab: openTab("a.sql"),
    });
    expect(s.tabs.map((t) => t.id)).toEqual(["a.sql"]);
    expect(s.activeId).toBe("a.sql");
    expect(s.tabs[0].saveStatus).toBe("clean");
  });

  test("re-opening an open tab just activates it (no duplicate)", () => {
    let s = withTabs("a.sql", "b.sql");
    s = sessionReducer(s, { type: "OPEN_TAB", tab: openTab("a.sql") });
    expect(s.tabs.map((t) => t.id)).toEqual(["a.sql", "b.sql"]);
    expect(s.activeId).toBe("a.sql");
  });
});

describe("CLOSE_TAB", () => {
  test("closing the active tab activates its right neighbor", () => {
    let s = withTabs("a.sql", "b.sql", "c.sql");
    s = sessionReducer(s, { type: "ACTIVATE_TAB", id: "b.sql" });
    s = sessionReducer(s, { type: "CLOSE_TAB", id: "b.sql" });
    expect(s.tabs.map((t) => t.id)).toEqual(["a.sql", "c.sql"]);
    expect(s.activeId).toBe("c.sql");
  });

  test("closing the last tab falls back to the left neighbor", () => {
    let s = withTabs("a.sql", "b.sql");
    s = sessionReducer(s, { type: "ACTIVATE_TAB", id: "b.sql" });
    s = sessionReducer(s, { type: "CLOSE_TAB", id: "b.sql" });
    expect(s.activeId).toBe("a.sql");
  });

  test("closing an inactive tab leaves activeId untouched", () => {
    let s = withTabs("a.sql", "b.sql");
    // active is b (last opened); close a.
    s = sessionReducer(s, { type: "CLOSE_TAB", id: "a.sql" });
    expect(s.activeId).toBe("b.sql");
  });

  test("closing the only tab clears activeId", () => {
    let s = withTabs("a.sql");
    s = sessionReducer(s, { type: "CLOSE_TAB", id: "a.sql" });
    expect(s.tabs).toEqual([]);
    expect(s.activeId).toBeNull();
  });
});

describe("REORDER_TABS", () => {
  test("moves a tab and ignores out-of-range / no-op moves", () => {
    let s = withTabs("a.sql", "b.sql", "c.sql");
    s = sessionReducer(s, { type: "REORDER_TABS", from: 0, to: 2 });
    expect(s.tabs.map((t) => t.id)).toEqual(["b.sql", "c.sql", "a.sql"]);
    const same = sessionReducer(s, { type: "REORDER_TABS", from: 1, to: 1 });
    expect(same).toBe(s);
    const oob = sessionReducer(s, { type: "REORDER_TABS", from: 5, to: 0 });
    expect(oob).toBe(s);
  });
});

describe("SET_STATUS / SET_ETAG", () => {
  test("updates a tab's save status + error and etag", () => {
    let s = withTabs("a.sql");
    s = sessionReducer(s, {
      type: "SET_STATUS",
      id: "a.sql",
      saveStatus: "error",
      error: "boom",
    });
    expect(s.tabs[0].saveStatus).toBe("error");
    expect(s.tabs[0].error).toBe("boom");
    s = sessionReducer(s, { type: "SET_ETAG", id: "a.sql", etag: "v2" });
    expect(s.tabs[0].etag).toBe("v2");
  });
});
