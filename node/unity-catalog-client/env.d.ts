/// <reference types="vite/client" />

// The package reads `import.meta.env.VITE_API_URL` as the optional base-URL
// override for the default Unity Catalog client (see src/api.ts). Declared here
// so the package type-checks standalone; the consuming app declares the same.
interface ImportMetaEnv {
  readonly VITE_API_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
