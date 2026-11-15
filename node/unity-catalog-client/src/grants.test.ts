import { expect, test } from "bun:test";
import {
  displayPrivilege,
  GRANTABLE_PRIVILEGES,
  normalizePrivilege,
  OWNER_PRIVILEGE,
} from "./grants.js";

test("normalizePrivilege accepts both wire spellings", () => {
  expect(normalizePrivilege("USE CATALOG")).toBe("USE_CATALOG");
  expect(normalizePrivilege(" use_catalog ")).toBe("USE_CATALOG");
  expect(normalizePrivilege("all  privileges")).toBe("ALL_PRIVILEGES");
});

test("displayPrivilege uses the OSS spelling", () => {
  expect(displayPrivilege("USE_CATALOG")).toBe("USE CATALOG");
  expect(displayPrivilege("SELECT")).toBe("SELECT");
});

test("OWNER is never grantable, and no type lists a privilege twice", () => {
  for (const [type, privileges] of Object.entries(GRANTABLE_PRIVILEGES)) {
    expect(privileges).not.toContain(OWNER_PRIVILEGE);
    expect(new Set(privileges).size, type).toBe(privileges.length);
    for (const p of privileges) expect(normalizePrivilege(p)).toBe(p);
  }
});

test("containers carry the privileges of what they contain", () => {
  expect(GRANTABLE_PRIVILEGES.catalog).toContain("SELECT");
  expect(GRANTABLE_PRIVILEGES.schema).toContain("READ_VOLUME");
  expect(GRANTABLE_PRIVILEGES.table).not.toContain("USE_SCHEMA");
});
