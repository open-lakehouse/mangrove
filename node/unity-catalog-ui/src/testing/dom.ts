// Registers a happy-dom window for the DOM tests.
//
// React DOM decides whether a DOM exists when it first loads, and Bun runs all
// test files in one process with a shared module cache. So the window has to
// exist before ANY test file loads React: the root `test:ui` script preloads
// this file (`--preload`). The DOM tests also import it first, so running a
// single file directly works without the flag.
//
// Server-rendering tests (renderToString) are unaffected by the window.

import { GlobalRegistrator } from "@happy-dom/global-registrator";

if (!GlobalRegistrator.isRegistered) {
  GlobalRegistrator.register({ url: "http://uc.test/" });
}
