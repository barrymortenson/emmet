// SPDX-License-Identifier: AGPL-3.0-or-later
import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import globals from "globals";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores(["**/dist/", "**/node_modules/", "target/", "apps/desktop/src-tauri/", "python/", "docs/"]),
  js.configs.recommended,
  tseslint.configs.recommended,
  { files: ["**/*.mjs", "**/*.js", "apps/desktop/vite.config.ts"], languageOptions: { globals: globals.node } },
]);
