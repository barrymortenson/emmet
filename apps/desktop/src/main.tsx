// SPDX-License-Identifier: AGPL-3.0-or-later
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { renderMode } from "./render";
import "./styles.css";

// Render mode's hidden window (ADR 0005) has no UI: it renders once and the host exits.
if (location.hash === "#render") {
  void renderMode();
} else {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
