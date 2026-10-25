// SPDX-License-Identifier: AGPL-3.0-or-later
import { StrictMode, useState } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

function App() {
  // The button is the E2E hello-world's target (ADR 0002) until the real UI replaces this window.
  const [greeted, setGreeted] = useState(false);
  return (
    <main>
      <h1>Emmet</h1>
      <p className="version">
        Version {__EMMET_VERSION__}
        {import.meta.env.DEV && <span className="dev">DEV</span>}
      </p>
      <button type="button" id="hello" onClick={() => setGreeted(true)}>
        Say hello
      </button>
      {greeted && <p id="greeting">Hello from Emmet</p>}
    </main>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
