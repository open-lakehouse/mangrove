import { describe, expect, test } from "bun:test";
import { languageOf, monacoLanguageOfPath } from "./language.js";

describe("monacoLanguageOfPath", () => {
  test.each([
    ["compose.yaml", "yaml"],
    ["envoy/envoy.yml", "yaml"],
    ["opa/data.json", "json"],
    ["trestle.toml", "ini"],
    ["init/seed.sh", "shell"],
    ["scripts/run.bash", "shell"],
    ["init/schema.sql", "sql"],
    ["README.md", "markdown"],
    ["images/app/Dockerfile", "dockerfile"],
    ["Dockerfile.dev", "dockerfile"],
    ["app.dockerfile", "dockerfile"],
    [".env", "ini"],
    ["secrets/.env.local", "ini"],
    ["services.env", "ini"],
    ["notes.txt", "plaintext"],
    ["LICENSE", "plaintext"],
    ["CONFIG.YAML", "yaml"],
  ])("%s → %s", (path, language) => {
    expect(monacoLanguageOfPath(path)).toBe(language);
  });

  test("leaves the session's classification alone", () => {
    expect(languageOf("compose.yaml")).toBe("plaintext");
    expect(languageOf("q.sql")).toBe("sql");
  });
});
