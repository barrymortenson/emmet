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
    "weathered-1": { k: 1, ty: 0 },
    "weathered-2": { k: 1, ty: 0 },
    finding: { k: 1, ty: 0 },
    stage1: { k: 0.9544, ty: 6.94 },
    stage2: { k: 0.9544, ty: 6.94 },
    stage3: { k: 0.9718, ty: 4.27 },
    empty: { k: 0.9086, ty: 14 },
  };

  // The model's own knobs: character, not raw dimensions (Barry 2026-10-27). The agent picks a few that change what
  // the model is like (how weathered, which features, another random arrangement); size, look, and color are build
  // settings or the prompt. These are the cottage template's own (crates/emmet-core/templates/cottage/model.py):
  // `weathering` lays that share of wall bricks in Reddish Brown by `random.Random(seed)`, and `roof` off shows the
  // room inside. Rendered at 20 % with seeds 1 and 2 (163 parts either way). Its width, depth, and wall height stay in
  // model.py for the rung 1 ladder and the tests, but the knobs panel leads with these.
  const params = [
    { name: "weathering", label: "Weathering", type: "int", value: 0, min: 0, max: 30, unit: "%", help: "how much old brick shows through the walls", group: "Look", line: 69 },
    { name: "chimney", label: "Chimney", type: "bool", value: true, help: "add a brick chimney at the back", group: "Features", line: 67 },
    { name: "roof", label: "Roof", type: "bool", value: true, help: "put the roof on (off shows the room inside)", group: "Features", line: 68 },
    { name: "seed", label: "Arrangement", type: "seed", value: 1, help: "another random arrangement of the same design", group: "Variation", line: 70 },
  ];


  // LDraw colors (LDConfig.ldr values) for the swatches. Part colors, not UI colors: the contrast check ignores them.
  const colors = {
    Tan: "#E4CD9E", "Dark Red": "#720E0F", White: "#FFFFFF", "Reddish Brown": "#582A12", Green: "#237841",
    "Light Bluish Gray": "#A0A5A9", "Dark Tan": "#958A73", Black: "#1B2A34", "Sand Blue": "#6074A1",
    "Dark Bluish Gray": "#646464", "Dark Blue": "#19325A", Red: "#B40000", Yellow: "#FAC80A",
    "Trans-Clear": "rgba(252, 252, 252, 0.35)",  // #FCFCFC, see-through
  };
  // What the color pickers offer: only colors the parts come in (section 14). Real availability arrives with 3B's
  // parts data pack; these are the colors the cottage's bricks and slopes are made in today.
  const colorChoices = {
    wall_color: ["Tan", "White", "Light Bluish Gray", "Dark Tan", "Sand Blue"],
    roof_color: ["Dark Red", "Black", "Reddish Brown", "Light Bluish Gray"],
  };

  // `emmet bom --json` on the cottage (163 parts in 29 kinds).
  const bom = [
    { part: "3004", name: "Brick 1 x 2", color: "Tan", qty: 55 },
    { part: "3004", name: "Brick 1 x 2", color: "Dark Bluish Gray", qty: 13 },
    { part: "3004", name: "Brick 1 x 2", color: "Reddish Brown", qty: 12 },
    { part: "3010", name: "Brick 1 x 4", color: "Dark Red", qty: 12 },
    { part: "3037", name: "Slope Brick 45 2 x 4", color: "Dark Red", qty: 12 },
    { part: "3005", name: "Brick 1 x 1", color: "Tan", qty: 10 },
    { part: "3004", name: "Brick 1 x 2", color: "Dark Red", qty: 6 },
    { part: "3039", name: "Slope Brick 45 2 x 2", color: "Dark Red", qty: 6 },
    { part: "60592", name: "Window 1 x 2 x 2 without Sill", color: "White", qty: 5 },
    { part: "60601", name: "Glass for Window 1 x 2 x 2 without Sill", color: "Trans-Clear", qty: 5 },
    { part: "3832", name: "Plate 2 x 10", color: "Tan", qty: 4 },
    { part: "2412b", name: "Tile 1 x 2 Grille with Groove", color: "Black", qty: 2 },
    { part: "3005", name: "Brick 1 x 1", color: "Dark Bluish Gray", qty: 2 },
    { part: "3041", name: "Slope Brick 45 2 x 4 Double", color: "Dark Red", qty: 2 },
    { part: "3062b", name: "Brick 1 x 1 Round with Hollow Stud", color: "Green", qty: 2 },
    { part: "3068b", name: "Tile 2 x 2 with Groove", color: "Light Bluish Gray", qty: 2 },
    { part: "3001", name: "Brick 2 x 4", color: "White", qty: 1 },
    { part: "3003", name: "Brick 2 x 2", color: "Dark Bluish Gray", qty: 1 },
    { part: "3043", name: "Slope Brick 45 2 x 2 Double", color: "Dark Red", qty: 1 },
    { part: "3062b", name: "Brick 1 x 1 Round with Hollow Stud", color: "Tan", qty: 1 },
    { part: "3062b", name: "Brick 1 x 1 Round with Hollow Stud", color: "Reddish Brown", qty: 1 },
    { part: "3068b", name: "Tile 2 x 2 with Groove", color: "White", qty: 1 },
    { part: "3068b", name: "Tile 2 x 2 with Groove", color: "Dark Blue", qty: 1 },
    { part: "3811", name: "Baseplate 32 x 32", color: "Green", qty: 1 },
    { part: "4032a", name: "Plate 2 x 2 Round with Axlehole", color: "Reddish Brown", qty: 1 },
    { part: "60596", name: "Door 1 x 4 x 6 Frame", color: "White", qty: 1 },
    { part: "60623", name: "Door 1 x 4 x 6 with 4 Panes and Stud Handle", color: "Reddish Brown", qty: 1 },
    { part: "6141", name: "Plate 1 x 1 Round", color: "Red", qty: 1 },
    { part: "6141", name: "Plate 1 x 1 Round", color: "Yellow", qty: 1 },
  ];


  // The cottage's 15 steps (`m.step(...)` names in model.py) and their part counts from placements.json.
  const steps = [
    ["Baseplate", 1], ["Walls, row 1", 17], ["Walls, row 2", 14], ["Walls, row 3", 20], ["Walls, row 4", 12],
    ["Walls, row 5", 15], ["Walls, row 6", 14], ["Inside", 9], ["Path and flowers", 6], ["Ceiling", 4],
    ["Roof, course 1", 18], ["Roof, course 2", 12], ["Roof, course 3", 6], ["Roof, course 4", 3], ["Chimney", 12],
  ].map(([name, parts], i) => ({ n: i + 1, name, parts }));
  // Which render shows the model built up to a step (note 6): the stage variants stop where these steps end.
  // The last step a partial build has reached, from its part count (36 parts is step 4).
  const stepOf = (parts) => { let sum = 0; for (const s of steps) { sum += s.parts; if (sum >= parts) return s.n; } return steps.length; };
  const stepRender = (n) => (n <= 4 ? "stage1" : n <= 9 ? "stage2" : n <= 12 ? "stage3" : "cottage");

  // `emmet check --json` on the cottage with its chimney started 9 plates up.
  const finding = {
    id: "F-0001", rule: "floating", severity: "error", parts: 9, partIds: "chimney/1–9", line: 253,
    message: "9 parts at z=9.5 aren't held by any stud: they touch nothing below them.",
    hint: "They start 9 plates above base (Baseplate 32 x 32), which ends at z=0.5: lower them by 9 plates, or put a part under them.",
    excerpt: ["            z = GROUND + 9", "            while z < ridge_top + 3:"],
  };

  // `emmet run --json` after an edit that passed an argument Model.add() doesn't take.
  const buildError = {
    code: "MODEL_EXEC_FAILED", line: 121,
    message: "TypeError: Model.add() got an unexpected keyword argument 'flip'",
    hint: "Fix model.py line 121 and run again.",
    // Emmet's excerpt is the call's first line; the mock shows the whole statement, lines 121–123.
    excerpt: ['frame = m.add(', '    "60592", "White", at=(*at, z), rot=turn, id=f"window/{name}", flip=True', ')'],
  };

  // `emmet run --json` on a copy of the project moved to another folder.
  const untrusted = {
    message: "Cottage isn't trusted on this machine (it was made elsewhere, or moved), so its model.py won't run until you trust it.",
    hint: "Read model.py first; it's code. Then trust the project in the app, or run emmet trust.",
  };

  const run = { parts: 163, steps: 15, ms: 99 };
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
      { t: "tool", icon: "cube", title: "Built 52 parts", detail: "Baseplate and walls, rows 1–3", status: "done", raw: 'run_model() → {"parts": 52, "steps": 4, "ms": 96}' },
      { t: "tool", icon: "check", title: "Checked: no problems", detail: "Nothing collides or floats", status: "done", raw: 'validate() → {"clean": true, "findings": []}' },
    ],
    stage2Running: [{ t: "tool", icon: "cube", title: "Building", detail: "Walls, rows 4–6, the room inside, and the path", status: "running", raw: "run_model()" }],
    stage2: [
      { t: "tool", icon: "cube", title: "Built 108 parts", detail: "Walls, the room inside, and the path", status: "done", raw: 'run_model() → {"parts": 108, "steps": 9, "ms": 97}' },
      { t: "tool", icon: "check", title: "Checked: no problems", detail: "Nothing collides or floats", status: "done", raw: 'validate() → {"clean": true}' },
    ],
    // A redirect with a link the user pasted. Opening it is the agent's own web tool, outside its sandbox, so it asks
    // (D-40). Part and color questions never need the web: Emmet's tools answer them (Barry 2026-10-27).
    pending: [{ t: "pending", text: "put the chimney at the back, like the stone one on this house: example.com/stone-cottage.jpg" }],
    delivered: [{ t: "user", text: "put the chimney at the back, like the stone one on this house: example.com/stone-cottage.jpg", delivered: true }],
    ack: [{ t: "agent", text: "Got it: the chimney goes at the back. I'd like to look at your photo first." }],
    approval: [{
      t: "approval", title: "Open a web page?", tool: "Web request",
      command: "example.com/stone-cottage.jpg",
      why: "You pasted this link. Claude wants to see the chimney in it before it builds the roof.",
    }],
    approvalDenied: [{ t: "note", text: "You said no to the web request. Claude carries on without it." }],
    stage3: [
      { t: "tool", icon: "cube", title: "Built 142 parts", detail: "Ceiling and roof, courses 1–2", status: "done", raw: 'run_model() → {"parts": 142, "steps": 12}' },
    ],
    finish: [
      { t: "tool", icon: "cube", title: "Built 163 parts", detail: "Roof ridge and chimney", status: "done", raw: 'run_model() → {"parts": 163, "steps": 15, "ms": 99}' },
      { t: "tool", icon: "check", title: "Checked: no problems", detail: "163 parts · 0 findings", status: "done", raw: 'validate() → {"clean": true, "findings": []}' },
      { t: "tool", icon: "eye", title: "Looked at the model", detail: "Iso render", status: "done", raw: 'render(view="iso")' },
      // The rung 1 transcript's closing line, with the template's numbers (163 parts) and knobs.
      { t: "agent", text: "The cottage is up: 10 x 8 studs, 6 bricks tall, 163 parts, 0 errors: a real door, glass in the windows, and a room inside with a bed, a table, and a stove. Knobs: weathering, roof on or off, and Shuffle." },
    ],
    // "Ask agent to fix" sends the finding in the form tests/replay/claude/derived-fix-collision.jsonl records.
    fixAsk: [{ t: "user", text: "Fix F-0001 (floating, parts chimney/1–8 at model.py:142): 8 parts at z=9.5 aren't held by any stud: they touch nothing below them. They start 9 plates above base (Baseplate 32 x 32), which ends at z=0.5: lower them by 9 plates, or put a part under them. Then run validate again.", auto: true }],
    fixing: [
      { t: "tool", icon: "check", title: "Found 1 problem", detail: "9 floating parts: the chimney", status: "done", raw: 'validate() → {"counts": {"floating": 1}}' },
      { t: "file", path: "model.py", summary: "Edited line 253", diff: ["-            z = GROUND + 9", "+            z = GROUND"] },
      { t: "tool", icon: "cube", title: "Building", detail: "Chimney from the baseplate up", status: "running", raw: "run_model()" },
    ],
    fixed: [
      { t: "tool", icon: "cube", title: "Built 163 parts", detail: "Chimney from the baseplate up", status: "done", raw: 'run_model() → {"parts": 163}' },
      { t: "tool", icon: "check", title: "Checked: no problems", detail: "163 parts · 0 findings", status: "done", raw: 'validate() → {"clean": true}' },
      { t: "agent", text: "Fixed F-0001: the chimney now starts on the baseplate at z=0.5. validate reports 0 findings." },
    ],
    // A request that led to the build error above (composed for this state).
    flipAsk: [{ t: "user", text: "flip the left window so its frame faces out" }],
    flipEdit: [
      { t: "file", path: "model.py", summary: "Edited line 122", diff: ['-                        "60592", "White", at=(*at, z), rot=turn, id=f"window/{name}"', '+                        "60592", "White", at=(*at, z), rot=turn, id=f"window/{name}", flip=True'] },
      { t: "tool", icon: "cube", title: "The build failed", detail: "model.py line 121", status: "error", raw: 'run_model() → MODEL_EXEC_FAILED' },
    ],
    // What "Rebuild with these settings" sends (composed for this state; no transcript has it yet).
    restyleAsk: [{ t: "user", text: "Rework the cottage for the new build settings: style Steampunk (was Town), detail Medium (was Low). Keep figure scale, 150–500 pieces, and the 32 × 32 baseplate, and keep its knobs.", auto: "settings" }],
    restyleThinking: [{ t: "thinking", text: "Brass and copper trim, pipes up the walls, a gear on the chimney, and tiles and frames for medium detail, still about 170 pieces." }],
    stopped: [{ t: "note", text: "You stopped the turn at 0:41. The model is as its last run left it: 108 parts." }],
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
  // Plain names, not brand or theme names (D-30). The values are the cottage's as built (163 parts, bricks, slopes, and a door and windows).
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
      // Small to large, in the hobby's own words where they aren't LEGO's names (D-30: "minifig" and Miniland are).
      options: [
        { id: "Micro", sub: "A whole building a few studs wide" },
        { id: "Micropolis", sub: "The community micro city standard: 16 × 16 modules, two-stud roads, a brick about 9 feet tall" },
        { id: "Midi", sub: "Between micro and figure scale: the shape and the main details" },
        { id: "Figure scale", sub: "Doors and rooms fit a 4-brick-tall figure, about 1:42" },
        { id: "Large", sub: "Bigger than figure scale, for display, like the large display cars" },
        { id: "Car 1:8", sub: "A large car with gears, panels, and working steering" },
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
      label: "Size", value: "Medium", help: "How many pieces the model should have",
      presets: [{ id: "Small", min: 50, max: 150 }, { id: "Medium", min: 150, max: 500 }, { id: "Large", min: 500, max: 1500 }],
    },
    base: { label: "Base", value: "32 × 32", help: "What it stands on (note 8)", options: ["None", "16 × 16", "32 × 32", "48 × 48"] },
    // LEGO's age bands (Barry 2026-10-27). No 1.5+: that's DUPLO, which the checks can't stack yet (plan session 8H).
    age: {
      label: "Age", value: "9+", help: "Who builds it",
      options: [
        { id: "4+", sub: "Big pieces, few steps" }, { id: "6+", sub: "More pieces, simple techniques" },
        { id: "9+", sub: "More complex techniques and steps" }, { id: "13+", sub: "Advanced techniques, fine detail" },
        { id: "18+", sub: "Display models for experienced builders" },
      ],
    },
  };

  return {
    brief, renders, params, colors, colorChoices, bom, steps, stepOf, stepRender, finding, buildError, untrusted, run, versions,
    agents, efforts, plans, usage, seg, PROMPT,
    project: { name: "Cottage", folder: "Documents/Emmet/Cottage" },
    imported: { name: "Cottage (imported)", file: "cottage.mpd", parts: 163, steps: 15 },
    recent: [
      { name: "Cottage", when: "today" },
      { name: "Station test", when: "yesterday" },
    ],
  };
})();
