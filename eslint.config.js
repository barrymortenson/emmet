// SPDX-License-Identifier: AGPL-3.0-or-later
import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import globals from "globals";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores(["**/dist/", "**/node_modules/", "target/", ".dev-data/", "apps/desktop/src-tauri/", "python/", "docs/", "**/web/vendor/"]),
  js.configs.recommended,
  tseslint.configs.recommended,
  { files: ["**/*.mjs", "**/*.js", "apps/desktop/vite.config.ts"], languageOptions: { globals: globals.node } },
  // The render spike's page runs in a webview, and the E2E specs in WebdriverIO's runner.
  { files: ["tests/spikes/headless-render/web/*.js"], languageOptions: { globals: globals.browser } },
  { files: ["tests/e2e/**/*.mjs"], languageOptions: { globals: { ...globals.node, ...globals.mocha, browser: "readonly" } } },
]);
