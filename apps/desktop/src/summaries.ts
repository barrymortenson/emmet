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
/** A number the result's text says after `before`: "Built 70 parts" → 70. */
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

/** A view's name for a sentence: "the front view". */
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
  Bash: { icon: "terminal", doing: "Running a command", done: () => "Ran a command", failed: "A command failed" },
  PowerShell: { icon: "terminal", doing: "Running a command", done: () => "Ran a command", failed: "A command failed" },
  shell: { icon: "terminal", doing: "Running a command", done: () => "Ran a command", failed: "A command failed" },
  commandExecution: { icon: "terminal", doing: "Running a command", done: () => "Ran a command", failed: "A command failed" },
  WebFetch: { icon: "external-link", doing: "Opening a web page", done: () => "Opened a web page", detail: (i) => host(str(i, "url")) },
  WebSearch: { icon: "search", doing: "Searching the web", done: () => "Searched the web" },
  TodoWrite: { icon: "list", doing: "Planning", done: () => "Made a plan" },
};

const DEFAULT: Words = { icon: "box", doing: "Working", done: () => "Used a tool" };

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

function words(name: string): Words {
  const n = toolName(name);
  return (n !== name ? EMMET[n] : CLI[name]) ?? EMMET[n] ?? DEFAULT;
}

/** A tool card: its icon, its title (what was done, or what it's doing while it runs), and the detail. */
export function toolCard(name: string, input: unknown, result: { ok: boolean; summary: string } | null): { icon: IconName; title: string; detail: string } {
  const w = words(name);
  const i = (input ?? {}) as Input;
  if (!result) return { icon: w.icon, title: w.doing, detail: w.detail?.(i, "") ?? "" };
  if (!result.ok) return { icon: w.icon, title: w.failed ?? `${w.doing} didn't work`, detail: "" };
  return { icon: w.icon, title: w.done(i, result.summary), detail: w.detail?.(i, result.summary) ?? "" };
}

/** What the status line says while `name` runs. */
export const doing = (name: string) => words(name).doing;

/** A project file in words: what it is ("the model") and its kind of change. models/door.py is the door sub-assembly. */
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
