// SPDX-License-Identifier: AGPL-3.0-or-later
// Settings → Agents (D-35): the OpenAI API key Codex runs on, kept in the OS keychain by the host. The window sends
// the key once and never reads it back; it only learns whether one is saved.
import { useEffect, useRef, useState } from "react";
import { commands } from "./bindings";

export function Settings({ onClose, onChanged }: { onClose: () => void; onChanged: () => void }) {
  const dialog = useRef<HTMLDialogElement | null>(null);
  const [saved, setSaved] = useState<boolean | null>(null);
  const [key, setKey] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    dialog.current?.showModal();
    void commands.openaiKeySaved().then(setSaved);
  }, []);

  const set = async (value: string | null) => {
    const r = await commands.openaiKeySet(value);
    if (r.status === "ok") {
      setSaved(r.data);
      setKey("");
      setError(null);
      onChanged();
    } else setError(`${r.error.message} ${r.error.hint}`.trim());
  };

  return (
    <dialog ref={dialog} className="settings" aria-labelledby="settings-title" onClose={onClose}>
      <h1 id="settings-title">Settings</h1>
      <h2>Agents</h2>
      <p>
        Codex runs in Emmet on your OpenAI API key, billed to your OpenAI account. Emmet keeps it in your system's
        keychain, never in a file.
      </p>
      <p id="openai-key-status">{saved == null ? "" : saved ? "An OpenAI API key is saved." : "No OpenAI API key yet."}</p>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (key.trim()) void set(key);
        }}
      >
        <label>
          OpenAI API key{" "}
          <input
            id="openai-key"
            type="password"
            autoComplete="off"
            spellCheck={false}
            value={key}
            onChange={(e) => setKey(e.target.value)}
          />
        </label>{" "}
        <button type="submit" disabled={!key.trim()}>
          Save
        </button>{" "}
        {saved && (
          <button type="button" onClick={() => void set(null)}>
            Remove
          </button>
        )}
      </form>
      <p className="muted">
        Claude Code uses whatever sign-in you gave it in your terminal: your plan, or an ANTHROPIC_API_KEY in your
        shell. Emmet never changes it.
      </p>
      {error && (
        <p className="notice" role="alert">
          {error}
        </p>
      )}
      <form method="dialog">
        <button type="submit">Close</button>
      </form>
    </dialog>
  );
}
