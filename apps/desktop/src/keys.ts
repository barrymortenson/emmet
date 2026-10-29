// SPDX-License-Identifier: AGPL-3.0-or-later
// The keyboard map (docs/design/keyboard.md), as one list of actions: the window's keys, macOS's menu bar (the host's
// menu.rs sends the same ids), Windows' title-row menus, and the shortcuts dialog all read it, so each action has one
// handler. A key that types a character never fires while a text field has focus.

export const windows = navigator.userAgent.includes("Windows");

/** An action's id: the same as the host's menu ids. */
export type Action =
  | "new"
  | "open"
  | "save"
  | "versions"
  | "export"
  | "settings"
  | "about"
  | "updates"
  | "cli"
  | "shortcuts"
  | "diagnostics"
  | "help:report"
  | "help:guide"
  | "undo-turn"
  | "undo-knob"
  | "redo-knob"
  | `view:${"iso" | "front" | "top" | "back" | "left" | "right"}`
  | "frame"
  | `tab:${"options" | "checks" | "parts" | "steps"}`
  | "step:prev"
  | "step:next"
  | "history"
  | "agent:message"
  | "agent:send"
  | "agent:stop"
  | "agent:fresh"
  | "agent:switch"
  | "agent:model"
  | "agent:usage"
  | "region:next"
  | "escape";

type Key = { key: string; mod?: boolean; alt?: boolean; shift?: boolean; /** Types a character: never in a text field. */ plain?: boolean };

/** Each shortcut, as keyboard.md lists it. */
export const KEYS: { action: Action; keys: Key[]; does: string; where: string }[] = [
  { action: "new", keys: [{ key: "n", mod: true }], does: "New project", where: "Anywhere" },
  { action: "open", keys: [{ key: "o", mod: true }], does: "Open a project or an LDraw file", where: "Anywhere" },
  { action: "save", keys: [{ key: "s", mod: true }], does: "Save version", where: "Anywhere" },
  { action: "export", keys: [{ key: "e", mod: true }], does: "Export", where: "Anywhere" },
  { action: "settings", keys: [{ key: ",", mod: true }], does: "Settings", where: "Anywhere" },
  { action: "agent:message", keys: [{ key: "l", mod: true }], does: "Go to the message box", where: "Anywhere" },
  ...(["iso", "front", "top", "back", "left", "right"] as const).map((v, i) => ({
    action: `view:${v}` as Action,
    keys: [{ key: String(i + 1), plain: true }],
    does: `View: ${v[0]!.toUpperCase()}${v.slice(1)}`,
    where: "Outside a text field",
  })),
  { action: "frame", keys: [{ key: "f", plain: true }], does: "Frame the selection, or the whole model", where: "Outside a text field" },
  { action: "step:prev", keys: [{ key: "[", plain: true }], does: "Previous step (opens Steps)", where: "Outside a text field" },
  { action: "step:next", keys: [{ key: "]", plain: true }], does: "Next step (opens Steps)", where: "Outside a text field" },
  ...(["options", "checks", "parts", "steps"] as const).map((t, i) => ({
    action: `tab:${t}` as Action,
    keys: [{ key: String(i + 1), mod: true }],
    does: `${t[0]!.toUpperCase()}${t.slice(1)}`,
    where: "Anywhere",
  })),
  { action: "undo-turn", keys: [{ key: "z", mod: true, alt: true }], does: "Undo the agent's last turn", where: "Anywhere" },
  { action: "undo-knob", keys: [{ key: "z", mod: true, plain: true }], does: "Undo a knob's draft before Keep", where: "The panel" },
  {
    action: "redo-knob",
    keys: windows ? [{ key: "y", mod: true, plain: true }] : [{ key: "z", mod: true, shift: true, plain: true }],
    does: "Redo a knob's draft",
    where: "The panel",
  },
  { action: "region:next", keys: [{ key: "F6" }], does: "Move between the viewer, the chat, and the panel", where: "Anywhere" },
  { action: "shortcuts", keys: [{ key: "/", mod: true }], does: "Keyboard shortcuts (this list)", where: "Anywhere" },
  { action: "escape", keys: [{ key: "Escape" }], does: "Close what's open; with nothing open, stop the agent", where: "Anywhere" },
];

/** A key as the platform writes it: ⌘S on macOS, Ctrl+S on Windows. */
export function keyName(k: Key): string {
  const name = k.key === "Escape" ? "Esc" : k.key.length === 1 ? k.key.toUpperCase() : k.key;
  if (windows) return [k.mod && "Ctrl", k.alt && "Alt", k.shift && "Shift", name].filter(Boolean).join("+");
  return `${k.alt ? "⌥" : ""}${k.shift ? "⇧" : ""}${k.mod ? "⌘" : ""}${name}`;
}

/** The keys an action has, for a menu item: "⌘S". */
export function keysFor(action: Action): string | undefined {
  const k = KEYS.find((r) => r.action === action)?.keys[0];
  return k ? keyName(k) : undefined;
}

const typing = (t: EventTarget | null) =>
  t instanceof HTMLElement && (t.isContentEditable || t.tagName === "TEXTAREA" || (t.tagName === "INPUT" && !["checkbox", "radio", "button", "range"].includes((t as HTMLInputElement).type)));

const CODES: Record<string, string> = { ",": "Comma", "/": "Slash", "[": "BracketLeft", "]": "BracketRight" };

/** The physical key's code: Option changes the character macOS reports (⌥⌘Z types Ω), never the code. */
const code = (key: string) => CODES[key] ?? (/^[a-z]$/.test(key) ? `Key${key.toUpperCase()}` : /^\d$/.test(key) ? `Digit${key}` : key);

/** The action a key press means here, or null. */
export function actionFor(e: KeyboardEvent): Action | null {
  const mod = windows ? e.ctrlKey : e.metaKey;
  for (const row of KEYS) {
    for (const k of row.keys) {
      // The physical key, or (without Option, which changes it) the character: some drivers give "," for Comma's code.
      const key = e.code === code(k.key) || (!e.altKey && e.key.toLowerCase() === k.key.toLowerCase());
      if (!key || !!k.mod !== mod || !!k.alt !== e.altKey || !!k.shift !== e.shiftKey) continue;
      if (k.plain && typing(e.target)) continue;
      return row.action;
    }
  }
  return null;
}
