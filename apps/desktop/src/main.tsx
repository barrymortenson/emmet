// SPDX-License-Identifier: AGPL-3.0-or-later
import { getCurrentWindow } from "@tauri-apps/api/window";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { renderMode } from "./render";
import "./styles.css";

// E2E builds keep every console error and uncaught error, so a spec can check the app logged none (plan section 21,
// 1D: "no console errors").
if (__EMMET_TEST_HOOKS__) {
  const errors: string[] = [];
  (window as unknown as { __emmetErrors: string[] }).__emmetErrors = errors;
  const original = console.error.bind(console);
  console.error = (...args: unknown[]) => {
    errors.push(args.map(String).join(" "));
    original(...args);
  };
  window.addEventListener("error", (e) => errors.push(`uncaught: ${e.message}`));
  window.addEventListener("unhandledrejection", (e) => errors.push(`unhandled rejection: ${String(e.reason)}`));
}

// Render mode's hidden window (ADR 0005) has no UI: it renders once and the host exits.
if (getCurrentWindow().label === "render") {
  void renderMode();
} else {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
