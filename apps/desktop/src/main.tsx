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
  // "ResizeObserver loop completed with undelivered notifications" names no one: each observer remembers where it
  // was made, and the error says which ones had just fired (the app's own lay out on the next frame; a library's may
  // not).
  const fired: { made: string; at: number }[] = [];
  const Observer = window.ResizeObserver;
  if (Observer) {
    window.ResizeObserver = class extends Observer {
      constructor(callback: ResizeObserverCallback) {
        const made = (new Error().stack ?? "").split("\n").slice(2, 5).join(" < ").replace(/\s+/g, " ");
        super((entries, observer) => {
          fired.push({ made, at: performance.now() });
          if (fired.length > 16) fired.shift();
          callback(entries, observer);
        });
      }
    };
  }
  window.addEventListener("error", (e) => {
    const loop = e.message.includes("ResizeObserver loop");
    // Floating UI's autoUpdate (under every Radix popover, menu, and tooltip) moves its element inside its own
    // observer's callback, by design, and WebView2 reports the loop notice for it (CI run 43483636244). That one
    // isn't the app's to fix; with only the app's observers firing, the notice is recorded.
    // The notice comes in the frame the observers fired in.
    const just = fired.filter((f) => performance.now() - f.at < 100).map((f) => f.made);
    if (loop && just.some((made) => made.includes("whileElementsMounted"))) return;
    errors.push(`uncaught: ${e.message}${loop ? ` (just fired: ${[...new Set(just)].join(" | ")})` : ""}`);
  });
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
