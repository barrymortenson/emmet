// SPDX-License-Identifier: AGPL-3.0-or-later
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Component, type ReactNode, StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { windows } from "./keys";
import { renderMode } from "./render";
import "./styles/tokens.css";
import "./styles/components.css";
import "./styles/approved.css";
import "./styles/app.css";

// The frozen design's window: the tokens' theme and platform on <html>, the base styles on <body> (components.md).
document.documentElement.dataset.platform = windows ? "windows" : "macos";
document.documentElement.dataset.theme ??= window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
document.body.classList.add("window");

// E2E builds keep every console error and uncaught error, so a spec can check the app logged none (plan section 21,
// 1D: "no console errors").
if (__EMMET_TEST_HOOKS__) {
  const errors: string[] = [];
  (window as unknown as { __emmetErrors: string[] }).__emmetErrors = errors;
  const original = console.error.bind(console);
  console.error = (...args: unknown[]) => {
    // With where it came from, so a spec can tell the app's errors from a test driver's.
    errors.push(`${args.map(String).join(" ")}\n${new Error().stack ?? ""}`);
    original(...args);
  };
  window.addEventListener("error", (e) => errors.push(`uncaught: ${e.message}`));
  window.addEventListener("unhandledrejection", (e) => errors.push(`unhandled rejection: ${String(e.reason)}`));
}

/** A fault anywhere in the window says so, instead of leaving it blank: what happened, and Reload. */
class AppBoundary extends Component<{ children: ReactNode }, { error: Error | null }> {
  state = { error: null as Error | null };
  static getDerivedStateFromError(error: Error) {
    return { error };
  }
  componentDidCatch(error: Error) {
    console.error("the window failed:", error);
  }
  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div className="alert-card" data-c="alert-card" role="alert" data-tone="bad" id="app-failed">
        <div className="alert-card-body">
          <b>Emmet's window stopped working</b>
          <p>Your projects are untouched. {this.state.error.message}</p>
          <div className="row">
            <button type="button" className="button" data-c="button" data-variant="primary" onClick={() => location.reload()}>
              Reload
            </button>
          </div>
        </div>
      </div>
    );
  }
}

// Render mode's hidden window (ADR 0005) has no UI: it renders once and the host exits.
if (getCurrentWindow().label === "render") {
  void renderMode();
} else {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <AppBoundary>
        <App />
      </AppBoundary>
    </StrictMode>,
  );
}
