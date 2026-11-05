// SPDX-License-Identifier: AGPL-3.0-or-later
// What the chat says the agent did, in plain words (session 4B): one table for every tool call, the product copy
// docs/design/content.md writes ("A tool card: what was done, in the past, then the detail"; "The agent's status: what
// it's doing now, in the present"). A card's closed line never shows a tool's name, a path, or code (law 2); the raw
// call is in the card, opened. Edits to the project's files are file cards, which say what changed in words.
import type { IconName } from "./ui";

type Input = Record<string, unknown>;

/** One tool's words: its icon, what the status says while it runs, and the card's title and detail once it's done. */
type Words = {
  icon: IconName;
  doing: string;
  done: (input: Input, result: string) => string;
  detail?: (input: Input, result: string) => string;
  /** The title when the call failed. */
  failed?: string;
};

const str = (i: Input, k: string) => (typeof i[k] === "string" ? (i[k] as string) : "");
const count = (n: number, one: string, many = `${one}s`) => `${n.toLocaleString("en-US")} ${n === 1 ? one : many}`;
/** The number `re`'s first group finds in the result's text: "Built 70 parts" → 70. */
const num = (text: string, re: RegExp) => {
  const m = text.match(re);
  return m ? Number(m[1]!.replaceAll(",", "")) : null;
};
const list = (v: unknown) => (Array.isArray(v) ? v.filter((x): x is string => typeof x === "string") : []);

/** validate's text in words: "No findings: the model is clean." or "1 error, 2 warnings: 1 floating, 2 collision." */
function checked(result: string): string {
  if (/^No (\w+ )?findings/.test(result)) return "Checked: no problems";
  const n = (num(result, /(\d+) errors?/) ?? 0) + (num(result, /(\d+) warnings?/) ?? 0) + (num(result, /(\d+) notes?/) ?? 0);
  return n ? `Found ${count(n, "problem")}` : "Checked the model";
}

const RULE_WORDS: Record<string, [string, string]> = {
  floating: ["floating part", "floating parts"],
  collision: ["overlap", "overlaps"],
  disconnected_submodel: ["loose sub-assembly", "loose sub-assemblies"],
  step_order: ["step out of order", "steps out of order"],
  step_size: ["step too big", "steps too big"],
  weak_connection: ["weak connection", "weak connections"],
  off_grid: ["part off the grid", "parts off the grid"],
  unknown_part: ["unknown part", "unknown parts"],
  part_unavailable: ["part never made", "parts never made"],
  color_unavailable: ["color never made", "colors never made"],
  obsolete_part: ["obsolete part", "obsolete parts"],
  alias_part: ["renamed part", "renamed parts"],
  figure_clearance: ["part in a figure's way", "parts in a figure's way"],
};

/** The problems by kind: "1 floating part · 2 overlaps". */
function problems(result: string): string {
  const head = result.split("\n")[0] ?? "";
  const after = head.split(": ")[1] ?? "";
  const kinds = [...after.matchAll(/(\d+) ([a-z_]+)/g)].map(([, n, r]) => {
    const [one, many] = RULE_WORDS[r!] ?? [r!.replaceAll("_", " "), r!.replaceAll("_", " ")];
    return count(Number(n), one, many);
  });
  return kinds.length ? kinds.join(" · ") : /^No /.test(result) ? "Nothing collides or floats" : "";
}

/** A view's name for a sentence: "Front view". */
const view = (i: Input) => (str(i, "view") ? `${str(i, "view")[0]!.toUpperCase()}${str(i, "view").slice(1)} view` : "");

/** Emmet's own tools (the `emmet` and `emmet_host` MCP servers). */
const EMMET: Record<string, Words> = {
  run_model: {
    icon: "box",
    doing: "Building",
    done: (_, r) => {
      const n = num(r, /Built ([\d,]+) parts?/);
      return n == null ? "Built the model" : `Built ${count(n, "part")}`;
    },
    detail: (_, r) => {
      const n = num(r, /in ([\d,]+) steps?/);
      return n == null ? "" : count(n, "step");
    },
    failed: "The build failed",
  },
  validate: { icon: "circle-check", doing: "Checking the model", done: (_, r) => checked(r), detail: (_, r) => problems(r) },
  search_parts: {
    icon: "search",
    doing: "Looking for parts",
    done: () => "Found the parts",
    detail: (i) => [str(i, "query"), ...list(i["more"])].filter(Boolean).join(" · "),
  },
  part_info: { icon: "search", doing: "Checking a part", done: () => "Checked a part" },
  render: { icon: "eye", doing: "Looking at the model", done: () => "Looked at the model", detail: (i) => view(i) },
  rules: { icon: "book-open", doing: "Reading Emmet's rules", done: () => "Read Emmet's rules", detail: () => "How to build with real parts" },
  api: { icon: "book-open", doing: "Reading how a model is written", done: () => "Read how a model is written" },
  guide: { icon: "book-open", doing: "Reading a build guide", done: () => "Read a build guide", detail: (i) => [str(i, "scale"), str(i, "style"), str(i, "subject")].filter(Boolean).join(" · ") },
  technique: { icon: "book-open", doing: "Looking up a technique", done: () => "Looked up a technique", detail: (i) => str(i, "name").replaceAll("_", " ") },
  get_params: { icon: "sliders-horizontal", doing: "Reading the knobs", done: () => "Read the knobs" },
  set_params: { icon: "sliders-horizontal", doing: "Setting the knobs", done: () => "Set the knobs" },
  list_colors: { icon: "search", doing: "Looking up colors", done: () => "Looked up colors" },
  bom: { icon: "list", doing: "Counting the parts", done: () => "Counted the parts" },
  steps: { icon: "list", doing: "Reading the steps", done: () => "Read the steps" },
  highlight: { icon: "eye", doing: "Pointing at parts", done: () => "Pointed at parts in the viewer" },
  camera: { icon: "eye", doing: "Turning the view", done: (i) => (str(i, "view") ? "Turned the view" : "Saw the view"), detail: (i) => view(i) },
  selection: { icon: "eye", doing: "Seeing what you picked", done: () => "Saw what you picked" },
  save_version: { icon: "save", doing: "Saving a version", done: () => "Saved a version", detail: (i) => str(i, "name") },
};

/** The CLIs' own tools that aren't file edits. */
const CLI: Record<string, Words> = {
  Read: { icon: "book-open", doing: "Reading the project", done: () => "Read the project", detail: (i) => fileWords(str(i, "file_path")).what },
  Glob: { icon: "search", doing: "Looking through the project", done: () => "Looked through the project" },
  Grep: { icon: "search", doing: "Looking through the project", done: () => "Looked through the project" },
  WebFetch: { icon: "external-link", doing: "Opening a web page", done: () => "Opened a web page", detail: (i) => host(str(i, "url")) },
  WebSearch: { icon: "search", doing: "Searching the web", done: () => "Searched the web" },
  TodoWrite: { icon: "list", doing: "Planning", done: () => "Made a plan" },
};

const DEFAULT: Words = { icon: "box", doing: "Working", done: () => "Used a tool" };

/** The tools that run a command on the user's computer (Claude Code's Bash and PowerShell, Codex's shell). */
const SHELLS = new Set(["Bash", "PowerShell", "shell", "commandExecution"]);

/** What a command does, in words (Barry 2026-11-04: "Ran a command" says nothing). The closed line never shows it. */
type Kind = { icon: IconName; doing: string; done: string; failed: string };
const KINDS = {
  install: { icon: "download", doing: "Installing a tool", done: "Installed a tool", failed: "Couldn't install a tool" },
  build: { icon: "box", doing: "Building", done: "Built the model", failed: "The build failed" },
  check: { icon: "circle-check", doing: "Checking the model", done: "Checked the model", failed: "Couldn't check the model" },
  parts: { icon: "search", doing: "Looking for parts", done: "Looked for parts", failed: "Couldn't look up the parts" },
  colors: { icon: "search", doing: "Looking up colors", done: "Looked up colors", failed: "Couldn't look up the colors" },
  look: { icon: "eye", doing: "Looking at the model", done: "Looked at the model", failed: "Couldn't look at the model" },
  count: { icon: "list", doing: "Counting the parts", done: "Counted the parts", failed: "Couldn't count the parts" },
  steps: { icon: "list", doing: "Reading the steps", done: "Read the steps", failed: "Couldn't read the steps" },
  export: { icon: "file-up", doing: "Exporting the model", done: "Exported the model", failed: "The export failed" },
  rules: { icon: "book-open", doing: "Reading Emmet's rules", done: "Read Emmet's rules", failed: "Couldn't read Emmet's rules" },
  try: { icon: "play", doing: "Trying the model", done: "Tried the model", failed: "The model didn't run" },
  math: { icon: "sliders-horizontal", doing: "Working out the sizes", done: "Worked out the sizes", failed: "Couldn't work out the sizes" },
  web: { icon: "download", doing: "Downloading", done: "Downloaded a file", failed: "The download didn't work" },
  versions: { icon: "rotate-ccw-clock", doing: "Looking at the versions", done: "Looked at the versions", failed: "Couldn't read the versions" },
  tidy: { icon: "folder-open", doing: "Tidying the project's files", done: "Tidied the project's files", failed: "Couldn't change the project's files" },
  read: { icon: "search", doing: "Looking through the project", done: "Looked through the project", failed: "Couldn't look through the project" },
  wait: { icon: "clock", doing: "Waiting", done: "Waited", failed: "Stopped waiting" },
} satisfies Record<string, Kind>;
/** Which kind wins when one command does several things: what the user would most want to hear about. */
const ORDER: (keyof typeof KINDS)[] = ["install", "export", "build", "try", "check", "look", "parts", "colors", "count", "steps", "rules", "web", "math", "versions", "tidy", "read", "wait"];

/** Words from the bench: while a command no one can read runs (chosen by the command, so they hold still), and in a
 * quiet stretch after a step (the dock's status, a new one every 15 seconds). */
const BENCH = ["Sorting parts", "Finding parts", "Choosing colors", "Counting studs", "Lining up the rows", "Snapping parts together"];
const UNKNOWN: Omit<Kind, "doing"> = { icon: "box", done: "Worked on the project", failed: "A step didn't work" };

const EMMET_CMD: Record<string, keyof typeof KINDS> = {
  run: "build", build: "build", check: "check", validate: "check", search: "parts", part: "parts", colors: "colors",
  render: "look", bom: "count", steps: "steps", export: "export", rules: "rules", api: "rules", guide: "rules", technique: "rules",
};
const READS = new Set(["ls", "cat", "head", "tail", "less", "more", "find", "tree", "grep", "rg", "ag", "wc", "file", "stat", "du", "pwd", "which", "diff", "jq", "awk", "sort", "uniq", "cut", "Get-ChildItem", "Get-Content", "Select-String", "dir", "type", "gci", "gc"]);
const TIDIES = new Set(["mkdir", "cp", "mv", "rm", "rmdir", "touch", "ln", "chmod", "tee", "Copy-Item", "Move-Item", "Remove-Item", "New-Item", "Set-Content"]);
const INSTALLERS = /^(pip3?|pipx|uv|npm|pnpm|yarn|brew|cargo|gem|winget|choco|apt(-get)?)$/;

/** The words of one simple command, without what wraps it: a variable set before it, "sudo", "time", "env", and the
 * program's folder and ".exe". ("cd x" is a command of its own once the line is split, and says nothing.) */
function program(segment: string): string[] {
  const w = segment.trim().split(/\s+/).filter(Boolean).map((t) => t.replace(/^["']|["']$/g, ""));
  while (w.length && (/^[A-Za-z_][A-Za-z0-9_]*=/.test(w[0]!) || ["sudo", "time", "env", "command", "exec", "nohup"].includes(w[0]!))) w.shift();
  if (w[0]) w[0] = w[0].replace(/^.*[\\/]/, "").replace(/\.exe$/i, "");
  return w;
}

function kindOf(w: string[]): keyof typeof KINDS | null {
  const [p, a = "", b = ""] = w;
  if (!p || ["cd", "echo", "printf", "true", "set", "export", "source", "."].includes(p)) return null;
  if (INSTALLERS.test(p) && /^(install|add|i|sync)$/.test(a === "pip" ? b : a)) return "install";
  if (p === "emmet") return EMMET_CMD[a] ?? null;
  if (p === "uv" && a === "run") return kindOf(w.slice(w[2] === "--" ? 3 : 2));
  if (/^(python[0-9.]*|py|pytest)$/.test(p)) return w.some((t) => /(^|[\\/])model\.py$|^models[\\/]|emmet_dsl/.test(t)) ? "try" : "math";
  if (/^(bc|expr|node|ruby|perl)$/.test(p)) return "math";
  if (/^(curl|wget|Invoke-WebRequest|iwr)$/.test(p)) return "web";
  if (p === "git") return "versions";
  if (p === "sed") return w.some((t) => /^-i/.test(t)) ? "tidy" : "read";
  if (READS.has(p)) return "read";
  if (TIDIES.has(p)) return "tidy";
  if (/^(sleep|Start-Sleep)$/.test(p)) return "wait";
  return null;
}

/** A shell's command as the user would hear it. A wrapped one ("/bin/zsh -lc 'ls'", Codex's way) is read inside. */
function commandWords(input: Input): Kind {
  let cmd = Array.isArray(input["command"]) ? list(input["command"]).join(" ") : str(input, "command");
  const wrapped = cmd.match(/^\S*?(?:ba|z)?sh(?:\.exe)?\s+-l?c\s+(['"])([\s\S]*)\1\s*$/);
  if (wrapped) cmd = wrapped[2]!;
  const kinds = cmd
    .replace(/<<-?\s*['"]?(\w+)['"]?[\s\S]*?\n\1\b/g, "")
    .split(/&&|\|\||[;|\n]/)
    .map((s) => kindOf(program(s)))
    .filter((k): k is keyof typeof KINDS => k != null);
  const k = ORDER.find((o) => kinds.includes(o));
  if (k) return KINDS[k];
  let h = 0;
  for (const c of cmd) h = (h * 31 + c.charCodeAt(0)) >>> 0;
  return { ...UNKNOWN, doing: benchWord(h) };
}

/** The `n`th of the bench's words, round and round. */
export const benchWord = (n: number) => BENCH[n % BENCH.length]!;

/** Edits, which show as file cards. */
const EDITS = new Set(["Edit", "MultiEdit", "Write", "NotebookEdit", "apply_patch", "fileChange"]);
/** Calls that are no step of the agent's: loading Emmet's deferred tools (ADR 0001), and a question (its card says it). */
const HIDDEN = new Set(["ToolSearch", "ask_user"]);

function host(url: string): string {
  try {
    return new URL(url).host;
  } catch {
    return "";
  }
}

/** A tool's own name, without the MCP server's prefix ("mcp__emmet__run_model" → "run_model"). */
export const toolName = (name: string) => name.replace(/^mcp__emmet(?:_host)?__/, "");

export const isEdit = (name: string) => EDITS.has(name);
export const isHidden = (name: string) => HIDDEN.has(toolName(name));

function words(name: string, input: Input = {}): Words {
  if (SHELLS.has(name)) {
    const k = commandWords(input);
    return { icon: k.icon, doing: k.doing, done: () => k.done, failed: k.failed };
  }
  const n = toolName(name);
  return (n !== name ? EMMET[n] : CLI[name]) ?? EMMET[n] ?? DEFAULT;
}

/** A tool card: its icon, its title (what was done, or what it's doing while it runs), and the detail. */
export function toolCard(name: string, input: unknown, result: { ok: boolean; summary: string } | null): { icon: IconName; title: string; detail: string } {
  const i = (input ?? {}) as Input;
  const w = words(name, i);
  if (!result) return { icon: w.icon, title: w.doing, detail: w.detail?.(i, "") ?? "" };
  if (!result.ok) return { icon: w.icon, title: w.failed ?? `${w.doing} didn't work`, detail: "" };
  return { icon: w.icon, title: w.done(i, result.summary), detail: w.detail?.(i, result.summary) ?? "" };
}

/** What the status line says while `name` runs with `input`. */
export const doing = (name: string, input?: unknown) => words(name, (input ?? {}) as Input).doing;

/** A project file in words: what it is ("the model") and the noun for it. models/door.py is the door sub-assembly. */
export function fileWords(path: string): { what: string; noun: string } {
  const rel = path.replaceAll("\\", "/").replace(/^.*?(?=(models\/|model\.py$|CARD\.md$|params\.json$|emmet\.toml$))/, "");
  if (rel === "model.py") return { what: "The model", noun: "the model" };
  if (rel === "CARD.md") return { what: "The model's notes", noun: "the model's notes" };
  if (rel === "params.json") return { what: "The knobs", noun: "the knobs" };
  if (rel === "emmet.toml") return { what: "The build settings", noun: "the build settings" };
  const sub = rel.match(/^models\/(?:.*\/)?([^/]+)\.py$/);
  if (sub) {
    const name = sub[1]!.replace(/[_-]+/g, " ");
    return { what: `The ${name} sub-assembly`, noun: `the ${name} sub-assembly` };
  }
  return { what: "A file in the project", noun: "a file in the project" };
}

/** A file card's title: "Changed the model", "Added the door sub-assembly". */
export function fileCard(path: string, change: "added" | "modified" | "removed"): string {
  const verb = change === "added" ? "Added" : change === "removed" ? "Removed" : "Changed";
  return `${verb} ${fileWords(path).noun}`;
}

/** The files an edit call names, for its file cards. */
export function editedFiles(name: string, input: unknown): string[] {
  const i = (input ?? {}) as Input;
  if (str(i, "file_path")) return [str(i, "file_path")];
  if (str(i, "notebook_path")) return [str(i, "notebook_path")];
  if (Array.isArray(i["files"])) return list(i["files"]);
  if (Array.isArray(i["changes"])) return (i["changes"] as Input[]).map((c) => str(c, "path")).filter(Boolean);
  return name === "apply_patch" ? [...String(i["patch"] ?? i["input"] ?? "").matchAll(/^\*\*\* (?:Add|Update|Delete) File: (.+)$/gm)].map((m) => m[1]!) : [];
}

/** A model's name for the composer's chip: "claude-opus-5-5" → "Opus 5.5"; Codex's ids are shown as they're named. */
export function modelName(id: string | null | undefined): string {
  if (!id) return "Default";
  const claude = id.match(/^claude-([a-z]+)-(\d+)(?:-(\d+))?/);
  if (claude) return `${claude[1]![0]!.toUpperCase()}${claude[1]!.slice(1)} ${claude[2]}${claude[3] ? `.${claude[3]}` : ""}`;
  return id;
}

/** An effort's name: "xhigh" → "Extra high". */
export function effortName(e: string | null | undefined): string {
  if (!e) return "Default";
  return { xhigh: "Extra high", max: "Max", ultra: "Ultra" }[e] ?? `${e[0]!.toUpperCase()}${e.slice(1)}`;
}
