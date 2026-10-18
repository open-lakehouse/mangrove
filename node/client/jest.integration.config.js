/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: "ts-jest",
  transform: { "^.+\\.ts$": ["ts-jest", { tsconfig: "tsconfig.test.json" }] },
  testEnvironment: "node",
  moduleDirectories: ["node_modules", "./dist"],
  moduleFileExtensions: ["js", "ts"],
  modulePathIgnorePatterns: ["<rootDir>/examples/"],
  testMatch: ["<rootDir>/__test__/integration/**/*.test.ts"],
  globalSetup: "<rootDir>/__test__/integration/setup.ts",
  globalTeardown: "<rootDir>/__test__/integration/teardown.ts",
  testTimeout: 30_000,
};
