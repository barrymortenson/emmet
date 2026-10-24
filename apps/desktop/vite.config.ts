// SPDX-License-Identifier: AGPL-3.0-or-later
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import pkg from "./package.json" with { type: "json" };

// Tauri loads the dev server from a fixed port and the build from dist/ (src-tauri/tauri.conf.json).
export default defineConfig({
  plugins: [react()],
  define: { __EMMET_VERSION__: JSON.stringify(pkg.version) },
  clearScreen: false,
  server: { port: 1420, strictPort: true },
});
