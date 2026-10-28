// SPDX-License-Identifier: AGPL-3.0-or-later
// Settings → Agents (D-35): what Codex runs on, either the user's ChatGPT plan (Sign in with ChatGPT, 1E) or an
// OpenAI API key, both kept in the OS keychain by the host. The window sends the key once and never reads it back; it
// only learns whether one is saved. Settings → Data (plan section 12): the parts data pack's date, Rebrickable's
// credit, and the parts list's color naming.
import { useEffect, useRef, useState } from "react";
import { commands, type DataSettings } from "./bindings";
import { CodexAccounts } from "./ChatGpt";
import { REBRICKABLE_CREDIT } from "./Parts";

const COLOR_NAMES: [string, string][] = [
  ["ldraw", "LDraw"],
  ["rebrickable", "Rebrickable"],
  ["bricklink", "BrickLink"],
];

export function Settings({ onClose, onChanged }: { onClose: () => void; onChanged: () => void }) {
  const dialog = useRef<HTMLDialogElement | null>(null);
  const [saved, setSaved] = useState<boolean | null>(null);
  const [key, setKey] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<DataSettings | null>(null);

  useEffect(() => {
    dialog.current?.showModal();
    void commands.openaiKeySaved().then(setSaved);
    void commands.dataSettings().then((r) => {
      if (r.status === "ok") setData(r.data);
      else setError(`${r.error.message} ${r.error.hint}`.trim());
    });
  }, []);

  const setColorNames = async (names: string) => {
    const r = await commands.setColorNames(names);
    if (r.status === "ok") setData((d) => (d ? { ...d, color_names: names } : d));
    else setError(`${r.error.message} ${r.error.hint}`.trim());
  };

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
      <h3>Codex</h3>
      <p>
        Codex runs in Emmet on your ChatGPT plan or on an OpenAI API key. Emmet itself is free and charges nothing; it
        keeps your sign-in and your key in your system's keychain, never in a file.
      </p>
      <CodexAccounts onChanged={onChanged} />
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
      <h3>Claude Code</h3>
      <p className="muted">
        Claude Code uses whatever sign-in you gave it in your terminal: your plan, or an ANTHROPIC_API_KEY in your
        shell. Emmet never changes it.
      </p>
      <h2>Data</h2>
      <p id="parts-data-status">
        {data == null
          ? ""
          : data.parts_data
            ? `Parts data from ${data.parts_data}.`
            : "No parts data yet: availability shows as Unknown."}
      </p>
      <label>
        Color names in the parts list{" "}
        <select
          id="color-names"
          value={data?.color_names ?? "ldraw"}
          disabled={data == null}
          onChange={(e) => void setColorNames(e.target.value)}
        >
          {COLOR_NAMES.map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
      </label>
      <p className="muted">{REBRICKABLE_CREDIT}</p>
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
