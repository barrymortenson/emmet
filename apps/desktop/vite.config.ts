// SPDX-License-Identifier: AGPL-3.0-or-later
import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import pkg from "./package.json" with { type: "json" };

const env = (globalThis as { process?: { env: Record<string, string | undefined> } }).process?.env ?? {};

// Tauri loads the dev server from a fixed port and the build from dist/ (src-tauri/tauri.conf.json).
export default defineConfig({
  plugins: [react()],
  // The viewer from its source, so the app (and just dev's hot reload) never runs a stale packages/viewer/dist.
  resolve: { alias: { "@emmet/viewer": fileURLToPath(new URL("../../packages/viewer/src/index.ts", import.meta.url)) } },
  define: {
    __EMMET_VERSION__: JSON.stringify(pkg.version),
    // `just e2e` builds with EMMET_E2E=1 so the specs can read the viewer's stats; no other build has the hooks.
    __EMMET_TEST_HOOKS__: JSON.stringify(env.EMMET_E2E === "1"),
  },
  clearScreen: false,
  server: { port: 1420, strictPort: true },
});
