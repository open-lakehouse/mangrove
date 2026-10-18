/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: "ts-jest",
  transform: { "^.+\\.ts$": ["ts-jest", { tsconfig: "tsconfig.test.json" }] },
  testEnvironment: "node",
  moduleDirectories: ["node_modules", "./dist"],
  moduleFileExtensions: ["js", "ts"],
  modulePathIgnorePatterns: ["<rootDir>/examples/"],
  testPathIgnorePatterns: ["/node_modules/", "<rootDir>/__test__/integration/"],
};
