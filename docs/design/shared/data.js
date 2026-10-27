// SPDX-License-Identifier: AGPL-3.0-or-later
// The content every 2A direction shows. Nothing here is lorem ipsum: the numbers, findings, errors, and parts come from
// `emmet run`, `check`, `bom`, and `params` on the cottage template and variants of it (docs/spec/sessions/2A.md), and
// the chat comes from the recorded transcripts (tests/replay/), the rung 1 eval (docs/evals/2026-10-26-rung1.md), and
// Barry's 1C session (docs/design/notes/). Where a line is composed for a state no transcript has yet, it says so.
window.EMMET_DATA = (() => {
  // Renders from `emmet render` (iso, 1400 px, shown at 1100). Each build stage frames its own bounds, so `k` and `ty`
  // (percent of the image) line its baseplate up with the finished model's: the camera never appears to move.
  const renders = {
    cottage: { k: 1, ty: 0 },
    "cottage-wide": { k: 1, ty: 0 },
    finding: { k: 1, ty: 0 },
    stage1: { k: 0.9626, ty: 5.77 },
    stage2: { k: 0.9641, ty: 5.44 },
    stage3: { k: 0.9801, ty: 3.08 },
    empty: { k: 0.9164, ty: 12.9 },
    "cottage-front": { k: 1, ty: 0 },
  };

  const params = [
    { name: "width", label: "Width", type: "int", value: 8, min: 6, max: 16, unit: "studs", help: "how many studs wide the house is", group: "Size", line: 41 },
    { name: "depth", label: "Depth", type: "int", value: 6, min: 4, max: 12, step: 2, unit: "studs", help: "how many studs deep the house is", group: "Size", line: 42 },
    { name: "wall_rows", label: "Wall height", type: "int", value: 6, min: 6, max: 9, unit: "bricks", help: "how many bricks tall the walls are", group: "Size", line: 43 },
    { name: "wall_color", label: "Wall color", type: "color", value: "Tan", help: "the color of the walls", group: "Colors", line: 44 },
    { name: "roof_color", label: "Roof color", type: "color", value: "Dark Red", help: "the color of the roof", group: "Colors", line: 45 },
    { name: "chimney", label: "Chimney", type: "bool", value: true, help: "add a brick chimney at the back", group: "Options", line: 46 },
  ];

  // LDraw colors (LDConfig.ldr values) for the swatches. Part colors, not UI colors: the contrast check ignores them.
  const colors = {
    Tan: "#E4CD9E", "Dark Red": "#720E0F", White: "#FFFFFF", "Reddish Brown": "#582A12", Green: "#237841",
    "Light Bluish Gray": "#A0A5A9", "Dark Tan": "#958A73", Black: "#1B2A34", "Sand Blue": "#6074A1",
  };
  // What the color pickers offer: only colors the parts come in (section 14). Real availability arrives with 3B's
  // parts data pack; these are the colors the cottage's bricks and slopes are made in today.
  const colorChoices = {
    wall_color: ["Tan", "White", "Light Bluish Gray", "Dark Tan", "Sand Blue"],
    roof_color: ["Dark Red", "Black", "Reddish Brown", "Light Bluish Gray"],
  };

  // `emmet bom --json` on the cottage (93 parts).
  const bom = [
    { part: "3004", name: "Brick 1 x 2", color: "Tan", qty: 48 },
    { part: "3005", name: "Brick 1 x 1", color: "Tan", qty: 12 },
    { part: "3004", name: "Brick 1 x 2", color: "Reddish Brown", qty: 11 },
    { part: "3037", name: "Slope Brick 45 2 x 4", color: "Dark Red", qty: 8 },
    { part: "3010", name: "Brick 1 x 4", color: "Dark Red", qty: 4 },
    { part: "3034", name: "Plate 2 x 8", color: "Tan", qty: 3 },
    { part: "60592", name: "Window 1 x 2 x 2 without Sill", color: "White", qty: 3 },
    { part: "3041", name: "Slope Brick 45 2 x 4 Double", color: "Dark Red", qty: 2 },
    { part: "3811", name: "Baseplate 32 x 32", color: "Green", qty: 1 },
    { part: "60596", name: "Door 1 x 4 x 6 Frame", color: "White", qty: 1 },
  ];

  // The cottage's 12 steps (`m.step(...)` names in model.py) and their part counts from placements.json.
  const steps = [
    ["Baseplate", 1], ["Walls, row 1", 11], ["Walls, row 2", 11], ["Walls, row 3", 13], ["Walls, row 4", 8],
    ["Walls, row 5", 10], ["Walls, row 6", 11], ["Ceiling", 3], ["Roof, course 1", 8], ["Roof, course 2", 4],
    ["Roof, course 3", 2], ["Chimney", 11],
  ].map(([name, parts], i) => ({ n: i + 1, name, parts }));
  // Which render shows the model built up to a step (note 6): the stage variants stop where these steps end.
  // The last step a partial build has reached, from its part count (36 parts is step 4).
  const stepOf = (parts) => { let sum = 0; for (const s of steps) { sum += s.parts; if (sum >= parts) return s.n; } return steps.length; };
  const stepRender = (n) => (n <= 4 ? "stage1" : n <= 8 ? "stage2" : n <= 10 ? "stage3" : "cottage");

  // `emmet check --json` on the cottage with its chimney started 9 plates up.
  const finding = {
    id: "F-0001", rule: "floating", severity: "error", parts: 8, partIds: "chimney/1–8", line: 142,
    message: "8 parts at z=9.5 aren't held by any stud: they touch nothing below them.",
    hint: "They start 9 plates above base (Baseplate 32 x 32), which ends at z=0.5: lower them by 9 plates, or put a part under them.",
    excerpt: ["            z = GROUND + 9", "            while z < ridge_top + 3:"],
  };

  // `emmet run --json` after an edit that passed an argument Model.add() doesn't take.
  const buildError = {
    code: "MODEL_EXEC_FAILED", line: 78,
    message: "TypeError: Model.add() got an unexpected keyword argument 'flip'",
    hint: "Fix model.py line 78 and run again.",
    excerpt: '                m.add("60592", "White", at=(x0, side_y, z), rot=2, id="window/left", flip=True)',
  };

  // `emmet run --json` on a copy of the project moved to another folder.
  const untrusted = {
    message: "Cottage isn't trusted on this machine (it was made elsewhere, or moved), so its model.py won't run until you trust it.",
    hint: "Read model.py first; it's code. Then trust the project in the app, or run emmet trust.",
  };

  const run = { parts: 93, steps: 12, ms: 92, wide: 124 };
  const versions = { emmet: "0.1.0", library: "2026-09", partsData: "2026-10-23", oldLibrary: "2026-06" };

  const agents = {
    claude: { name: "Claude", cli: "Claude Code", version: "2.1.310", plan: "Max plan", models: ["Opus 5.5", "Sonnet 5.5", "Haiku 5.5"], model: "Opus 5.5" },
    codex: { name: "Codex", cli: "Codex", version: "0.164.0", plan: "ChatGPT plan", models: ["GPT-6.1 Sol", "GPT-6 Astra"], model: "GPT-6.1 Sol" },
  };
  const efforts = ["Low", "Medium", "High"];
  const plans = {
    claude: "Claude Code needs a Claude Pro, Max, Team, or Enterprise plan, or a Claude Console account. The free Claude plan doesn't include it.",
    codex: "Codex comes with every ChatGPT plan, Free included, or runs on an OpenAI API key.",
  };
  // Plan usage, as plan section 14's layout brief shows it (Claude Code's own report; D-36).
  const usage = [
    { label: "Current session", pct: 16, reset: "14m" },
    { label: "Weekly, all models", pct: 4, reset: "6d 9h" },
    { label: "Weekly, Fable", pct: 0, reset: "6d 9h" },
  ];

  // The rung 1 prompt (docs/spec/ladder.md, tests/replay/claude/derived-rung1.jsonl).
  const PROMPT = "Build a small cottage on a green 32 × 32 baseplate: four walls with a door and some windows, a sloped roof, and an optional chimney. Give me sliders for the width, the depth, the wall height, the wall color, the roof color, and whether it has a chimney.";
  const QUESTION = {
    t: "question", header: "Style", question: "Which style should the cottage have?",
    options: [
      { label: "Classic: tan walls and a dark red roof", description: "Recommended", recommended: true },
      { label: "Modern: white walls and a black roof", description: "" },
    ],
  };

  // Chat segments, in the order the rung 1 transcripts show: the agent asks, reads the rules, searches, writes
  // model.py, then builds and checks in stages (the eval: "built the cottage in stages, each run, validated, and
  // rendered"). Tool titles are the plain-language summaries 4B builds; `raw` is what the expanded card shows.
  const seg = {
    prompt: [{ t: "user", text: PROMPT }],
    thinking: [{ t: "thinking", text: "Planning the cottage: the baseplate, then walls in rows with a door and windows, a ceiling, a gable roof, and the chimney." }],
    question: [QUESTION],
    answered: [{ ...QUESTION, answered: "Classic: tan walls and a dark red roof" }],
    start: [
      { t: "tool", icon: "book", title: "Read Emmet's rules", detail: "How to build with real parts", status: "done", raw: "rules()" },
      { t: "tool", icon: "search", title: "Found the parts", detail: "brick 1 x 2 · slope 45 2 x 4 · door 1 x 4 x 6 · window 1 x 2 x 2", status: "done", raw: 'search_parts(["brick 1x2", "slope 45 2x4", "door 1x4x6", "window 1x2x2"]) → 12 results' },
      { t: "file", path: "model.py", summary: "Wrote the model · 144 lines", diff: ["+@model", "+def cottage(", "+    width: Annotated[int, Range(6, 16)] = 8,", "+    depth: Annotated[int, Range(4, 12), Step(2)] = 6,", "+    wall_rows: Annotated[int, Range(6, 9)] = 6,", "+    …"] },
    ],
    stage1Running: [{ t: "tool", icon: "cube", title: "Building", detail: "Baseplate and walls, rows 1–3", status: "running", raw: "run_model()" }],
    stage1: [
      { t: "tool", icon: "cube", title: "Built 36 parts", detail: "Baseplate and walls, rows 1–3", status: "done", raw: 'run_model() → {"parts": 36, "steps": 4, "ms": 89}' },
      { t: "tool", icon: "check", title: "Checked: no problems", detail: "Nothing collides or floats", status: "done", raw: 'validate() → {"clean": true, "findings": []}' },
    ],
    stage2Running: [{ t: "tool", icon: "cube", title: "Building", detail: "Walls, rows 4–6, and the ceiling", status: "running", raw: "run_model()" }],
    stage2: [
      { t: "tool", icon: "cube", title: "Built 68 parts", detail: "Walls and ceiling", status: "done", raw: 'run_model() → {"parts": 68, "steps": 8, "ms": 88}' },
      { t: "tool", icon: "check", title: "Checked: no problems", detail: "Nothing collides or floats", status: "done", raw: 'validate() → {"clean": true}' },
    ],
    pending: [{ t: "pending", text: "put the chimney at the back, please" }],
    delivered: [{ t: "user", text: "put the chimney at the back, please", delivered: true }],
    ack: [{ t: "agent", text: "Got it: the chimney goes at the back, on the ridge line. Roof next." }],
    approval: [{
      t: "approval", title: "Open a web page?", tool: "Web request",
      command: "rebrickable.com/parts/3037/slope-45-2-x-4",
      why: "To check which colors Slope 45 2 x 4 comes in before the roof uses Dark Red.",
    }],
    approvalDenied: [{ t: "note", text: "You said no to the web request. Claude carries on without it." }],
    stage3: [
      { t: "tool", icon: "cube", title: "Built 80 parts", detail: "Roof, courses 1–2", status: "done", raw: 'run_model() → {"parts": 80, "steps": 10}' },
    ],
    finish: [
      { t: "tool", icon: "cube", title: "Built 93 parts", detail: "Roof ridge and chimney", status: "done", raw: 'run_model() → {"parts": 93, "steps": 12, "ms": 92}' },
      { t: "tool", icon: "check", title: "Checked: no problems", detail: "93 parts · 0 findings", status: "done", raw: 'validate() → {"clean": true, "findings": []}' },
      { t: "tool", icon: "eye", title: "Looked at the model", detail: "Iso render", status: "done", raw: 'render(view="iso")' },
      { t: "agent", text: "The cottage is up: 8 x 6 studs, 6 bricks tall, 93 parts, 0 errors. Sliders: width, depth, wall rows, wall color, roof color, chimney." },
    ],
    // "Ask agent to fix" sends the finding in the form tests/replay/claude/derived-fix-collision.jsonl records.
    fixAsk: [{ t: "user", text: "Fix F-0001 (floating, parts chimney/1–8 at model.py:142): 8 parts at z=9.5 aren't held by any stud: they touch nothing below them. They start 9 plates above base (Baseplate 32 x 32), which ends at z=0.5: lower them by 9 plates, or put a part under them. Then run validate again.", auto: true }],
    fixing: [
      { t: "tool", icon: "check", title: "Found 1 problem", detail: "8 floating parts: the chimney", status: "done", raw: 'validate() → {"counts": {"floating": 1}}' },
      { t: "file", path: "model.py", summary: "Edited line 142", diff: ["-            z = GROUND + 9", "+            z = GROUND"] },
      { t: "tool", icon: "cube", title: "Building", detail: "Chimney from the baseplate up", status: "running", raw: "run_model()" },
    ],
    fixed: [
      { t: "tool", icon: "cube", title: "Built 93 parts", detail: "Chimney from the baseplate up", status: "done", raw: 'run_model() → {"parts": 93}' },
      { t: "tool", icon: "check", title: "Checked: no problems", detail: "93 parts · 0 findings", status: "done", raw: 'validate() → {"clean": true}' },
      { t: "agent", text: "Fixed F-0001: the chimney now starts on the baseplate at z=0.5. validate reports 0 findings." },
    ],
    // A request that led to the build error above (composed for this state).
    flipAsk: [{ t: "user", text: "flip the left window so its frame faces out" }],
    flipEdit: [
      { t: "file", path: "model.py", summary: "Edited line 78", diff: ['-                m.add("60592", "White", at=(x0, side_y, z), rot=2, id="window/left")', '+                m.add("60592", "White", at=(x0, side_y, z), rot=2, id="window/left", flip=True)'] },
      { t: "tool", icon: "cube", title: "The build failed", detail: "model.py line 78", status: "error", raw: 'run_model() → MODEL_EXEC_FAILED' },
    ],
    // What "Rebuild with these settings" sends (composed for this state; no transcript has it yet).
    restyleAsk: [{ t: "user", text: "Rework the cottage for the new build settings: style Medieval (was Town), detail Medium (was Low). Keep figure scale, 50–150 pieces, and the 32 × 32 baseplate, and keep its sliders.", auto: "settings" }],
    restyleThinking: [{ t: "thinking", text: "Stone walls with timber framing above the first rows, window frames and tiles for medium detail, still about 120 pieces." }],
    stopped: [{ t: "note", text: "You stopped the turn at 0:41. The model is as its last run left it: 68 parts." }],
    crashed: [{ t: "crash", title: "Claude Code stopped unexpectedly", text: "It exited with code 1 in the middle of a turn. Your project is untouched: model.py and the last build are as they were." }],
    // Barry's request from his 1C session (docs/design/notes/emmet-1c-claude.png), sent to Codex after a switch.
    garage: [{ t: "user", text: "make the garage and cottage one building, make it larger and make a door from the cottage to the garage, make the garage fit two cars." }],
    codexStart: [
      { t: "tool", icon: "book", title: "Read the project", detail: "model.py · CARD.md · params.json", status: "done", raw: "read model.py, CARD.md" },
      { t: "thinking", text: "Joining the garage to the cottage's right wall and widening it to fit two cars." },
    ],
  };

  // Build settings (Barry 2026-10-27, a proposal for 2A's direction B): what to build, rather than the model's own
  // dimensions. They steer the agent: they're kept with the project, sent with every message, and changing one asks
  // the agent to rework the model (a turn on the user's plan), unlike a model tweak, which reruns model.py at once.
  // Plain names, not brand or theme names (D-30). The values are the cottage's as built (93 parts, bricks and slopes).
  const brief = {
    style: {
      label: "Style", value: "Town", help: "The look the agent builds in",
      options: [
        { id: "Town", sub: "Houses, shops, streets" }, { id: "Medieval", sub: "Stone, timber, castles" },
        { id: "Blocky", sub: "Voxel art, one-stud cubes" }, { id: "Dinosaur park", sub: "Jungle, fences, big animals" },
        { id: "Space", sub: "Bases, rovers, gray and blue" }, { id: "Pirate", sub: "Ships, docks, islands" },
        { id: "Modern", sub: "Glass, clean lines" }, { id: "Fantasy", sub: "Towers, magic, color" },
      ],
    },
    scale: {
      label: "Scale", value: "Figure scale", help: "How big the world is",
      options: [
        { id: "Figure scale", sub: "Doors and rooms fit a 4-brick-tall figure, about 1:42" },
        { id: "Micro", sub: "A whole building a few studs wide" },
        { id: "Mini town", sub: "Half figure scale, about 1:80, for a big city on a table" },
        { id: "Display", sub: "Large and detailed, no figures to fit" },
        { id: "Car 1:8", sub: "A large car model with gears, panels, and a working steering" },
      ],
    },
    detail: {
      label: "Detail", value: "Low", help: "How much the agent adds beyond the shape",
      options: [
        { id: "Low", sub: "Bricks, plates, and slopes: quick to build" },
        { id: "Medium", sub: "Plus tiles, trim, and window frames" },
        { id: "High", sub: "Plus textures, sideways building, and small details" },
      ],
    },
    size: {
      label: "Target size", min: 50, max: 150, help: "How many pieces the model should have",
      presets: [{ id: "Small", min: 50, max: 150 }, { id: "Medium", min: 150, max: 500 }, { id: "Large", min: 500, max: 1500 }],
    },
    base: { label: "Baseplate", value: "32 × 32", help: "What it stands on (note 8)", options: ["None", "16 × 16", "32 × 32", "48 × 48"] },
  };

  return {
    brief, renders, params, colors, colorChoices, bom, steps, stepOf, stepRender, finding, buildError, untrusted, run, versions,
    agents, efforts, plans, usage, seg, PROMPT,
    project: { name: "Cottage", folder: "Documents/Emmet/Cottage" },
    imported: { name: "Cottage (imported)", file: "cottage.mpd", parts: 93, steps: 12 },
    recent: [
      { name: "Cottage", when: "today" },
      { name: "Station test", when: "yesterday" },
    ],
  };
})();
