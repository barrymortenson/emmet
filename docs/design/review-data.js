// SPDX-License-Identifier: AGPL-3.0-or-later
// What docs/design/review.html says: the three directions, how each answers Barry's notes, one row per screen or
// group of states with each direction's handling and the trade-off, the recommendation, and the contrast results.
window.REVIEW_DIRS = [
  { id: "a", name: "A · Workbench", summary: "Plan section 14's layout brief, done well: the project tree on the left, the viewer with a tabbed drawer under it, the chat at full height on the right with what the agent is doing pinned at its top, and a status bar. Calm, fairly dense, native.", best: "Best for: long sessions by people who like every panel in view. Risk: the model gets about a third of the window." },
  { id: "b", name: "B · Canvas", summary: "The render fills the window edge to edge; everything else floats on it as rounded panels at the edges. A capsule at the top center says what the agent is doing; the chat is one line above the composer, with the full history as a side sheet. Warm, roomy, friendly.", best: "Best for: people new to agents and to CAD, the users D-41 is about. Risk: floating panels need care at small window sizes." },
  { id: "c", name: "C · Command", summary: "A command bar across the top is the composer and the palette, with the live status at its right end; questions and approvals dock under it. The viewer in the center, an inspector on the right, and the turn as a timeline of chips along the bottom. Dense, keyboard-first, precise.", best: "Best for: power users and long agent turns. Risk: the most to learn, and a one-line bar is a cramped place for a long first prompt." },
];

window.REVIEW_NOTES = [
  { note: "1 · What the agent is doing, in context", a: "A pinned strip at the top of the chat (spinner, line, time, Stop · Esc), repeated in the status bar and the title's build pill.", b: "A capsule at the top center, the first thing the eye hits; it turns to an attention color when the agent waits on you.", c: "The right end of the command bar, plus the last chip of the timeline." },
  { note: "2 · Less code in the chat", a: "Plain-language tool cards, raw output folded; a finished run of cards folds to one line (\"11 steps · built in 4 stages\").", b: "One line above the composer by default; the history sheet folds finished runs the same way.", c: "Chips instead of cards; code only in the transcript drawer, folded." },
  { note: "3 · Agent picker and Start fresh, like nurb", a: "In the chat header, exactly nurb's place; switching shows a divider.", b: "In the composer's top row and in the history sheet's header.", c: "In the command bar, beside the model chip." },
  { note: "4 · Model and effort picker", a: "The chip beside Send; a popover with the plan note.", b: "The chip beside Send; a card at the bottom right, so it never covers the house.", c: "A chip in the command bar; a popover under it." },
  { note: "5 · Sliders worth having", a: "The drawer's Parameters tab: Size (two across), Colors, Options; help lines from the docstring; \"was 8\" and Keep as default.", b: "The left card, one column, the same groups and words.", c: "Compact rows in the inspector, the same groups and words." },
  { note: "6 · A step shows the build up to it", a: "The Steps tab and the scrubber; later steps grayed in the tree too.", b: "The left card's Steps list and the scrubber at the top right.", c: "The scrubber names the step (\"step 8/12 Ceiling\"); the inspector's list highlights it." },
];

const ROW = (screen, title, a, b, c, trade, also) => ({ screen, title, a, b, c, trade, also });
window.REVIEW_ROWS = [
  ROW("built", "The workspace, built and clean",
    "The viewer is the largest region (about 700 × 500), framed by the tree, the drawer, and the chat. Everything is one glance away.",
    "The model fills the window; the left card, the capsule, and the composer float at the edges.",
    "The viewer takes the center; the inspector on the right and the timeline at the bottom stay thin.",
    "Law 1 says the viewer is the hero. B gives it the most room, C next, A the least; A shows the most at once."),
  ROW("fr-agents", "First run",
    "A setup assistant: steps on the left, one task a page, agents installed and signed in by click, the plan text and the provider notice on the same page.",
    "One big friendly card over a live render that grows behind it, so the user sees what Emmet does before reading a word.",
    "A setup checklist with a plain-words progress log; nothing asks the user to type, and the command bar is optional.",
    "A is the most familiar, B the most inviting, C the most transparent about what's happening (and a little technical for first-timers).",
    ["fr-welcome", "fr-library", "fr-start"]),
  ROW("new-empty", "Empty: a new model from a prompt",
    "The empty baseplate fills the viewer; the chat's empty state says what happens next, and the prompt goes in the chat's composer.",
    "An empty baseplate, a tip above the composer with the prompt filled in; the left card says sliders come after the first build.",
    "The prompt goes in the top bar over a bare baseplate; the inspector says what will appear.",
    "B and A give a long first prompt room; C's one-line bar is cramped for it.", ["thinking"]),
  ROW("building", "Agent status while it builds (and the build animation)",
    "The now strip at the top of the chat, the status bar, and the title pill; parts drop in with \"Placing parts in build order · Skip\" over the viewer.",
    "The capsule at the top center, the running tool above the composer, and Skip with the scrubber at the top right (\"Step 4/12\").",
    "The bar's right end (spinner, line, time, Stop · Esc), the outlined running chip, and Skip with \"step 4/12 Walls, row 3\".",
    "B answers \"what is it doing?\" first; A puts it where the conversation is; C keeps it in one fixed spot that depends on the user glancing up."),
  ROW("question", "Question waiting for an answer",
    "The question card in the chat: numbered answers and Other; the strip and status bar say it's waiting on you.",
    "The card docks at the bottom right; the capsule turns attention-colored.",
    "Docked under the command bar, keyboard-first (1-3, Other inline); the viewer moves down about 70 px.",
    "A reads like Claude Code; B keeps the model clear but separates the card from the composer; C is the fastest by keyboard."),
  ROW("redirect", "Redirect pending",
    "A pending bubble at the end of the conversation: Edit, Cancel, Stop and send now.",
    "The pending bubble docks at the right beside the composer, the running tool above it.",
    "A \"Queued\" dock under the bar; the bar's placeholder becomes \"Redirect Claude: it reads this at its next step\".",
    "A keeps it in the conversation's flow but it can scroll; B and C keep it in view."),
  ROW("approval", "Waiting for approval",
    "The approval card in the chat (the page, why, Allow once, for this session, Don't allow); the strip turns to attention.",
    "The same card in the bottom-right corner; the capsule says what it's waiting for.",
    "The same dock as a question, with numbered choices.",
    "C has the least room for a long command; A the most."),
  ROW("sliders", "Tweak with sliders",
    "The drawer's Parameters tab: Size two across, Colors, Options; a moved slider shows \"was 8\" and a Keep as default bar.",
    "The left card, one column; the moved row gets an accent bar and \"was 8\"; the agent's stale line hides while changes are unsaved.",
    "Compact inspector rows with a \"was 8\" chip and a Keep / Reset box at the top.",
    "A has the most room for words but takes viewer height; B's card scrolls; C's rows are dense for a touchpad."),
  ROW("steps", "Steps: the build up to step 8",
    "The Steps tab lists all 12 with part counts; later steps are grayed in the list and the tree; the viewer shows only what's built.",
    "The left card's Steps list and the scrubber agree, on opposite sides of the window.",
    "A single-column list in the inspector; the scrubber names the step.",
    "All three show the model up to the step (note 6); A says it in the most places."),
  ROW("findings", "Checks with findings, and the fix",
    "A pin beside the chimney, the Checks tab with the plain sentence, model.py lines 142-143, and Ask agent to fix.",
    "A larger pin with the reason (\"8 parts float · the chimney starts 9 plates up\"), the finding in the left card, and Ask agent to fix above the composer too.",
    "A pin labeled \"F-0001 · 8 floating\", the finding in the inspector; while fixing, the button becomes \"Claude is fixing this\".",
    "B makes problems the most visible; C fits the most into the least space; A's pin is small at its viewer size.", ["fixing"]),
  ROW("failed", "Build failed",
    "The error card in the chat (the TypeError, line 78); a note on the viewer and the title pill say the last good model is shown.",
    "A red pill on the viewer, a red card head, and the error docked at the right with the line.",
    "A red dock under the bar with the line and Ask agent to fix; the viewer keeps the last good model.",
    "All keep the last good model (law 1). C's dock takes viewer height until it's fixed."),
  ROW("crashed", "Stopped by the user, and agent crashed",
    "A crash card in the chat with Restart Claude Code; the strip and status bar turn red. Stopped is a quiet note in the chat.",
    "The crash card docks at the right and the capsule turns red; a stop note sits above the composer.",
    "A red dock with Restart; the interrupted chip reads \"cut off\". Stopped is a quiet status and a note chip.",
    "All say the project is untouched. B and C put the action away from the conversation.", ["stopped"]),
  ROW("signed-out", "Agent not signed in",
    "The chat becomes the sign-in card: Sign in to Claude, or Use Codex instead; the viewer keeps working.",
    "The setup card docks at the right; the capsule is warn-colored.",
    "An amber dock under the bar with the same two buttons and the plan text.",
    "A can't be missed; C is the least warm."),
  ROW("switch-agent", "Switch agent mid-project",
    "The picker drops from the chat header like nurb's; after the switch, a divider and a fresh conversation.",
    "The history sheet opens with the picker in its header; the model slides left to stay clear.",
    "The picker drops from the bar's agent button over the inspector; the switched state opens the transcript with its divider.",
    "A is exactly note 3. B moves the model to make room, the one time it does.", ["switched"]),
  ROW("model-picker", "Model and effort",
    "A popover from the chip beside Send, with \"Bigger models and higher effort use up your plan faster.\"",
    "A bottom-right card, because above the composer it covered the house.",
    "A popover under the bar's model chip.",
    "A keeps it beside Send, as nurb does."),
  ROW("usage", "Plan usage",
    "Meters in the status bar; a popover with each window and its reset time.",
    "Meters in the top-right pill; a popover under them.",
    "Meters in the status bar; a popover at the bottom right.",
    "Quiet in all three, as it should be until the 90% line (4B)."),
  ROW("export", "Export",
    "Export on the Parts tab and in the title bar; one menu with parts lists, the model, and instructions.",
    "A menu under the top-right Export button, with the Parts tab open in the left card.",
    "Parts tab, Export, then a menu over the inspector; also /export in the bar.",
    "The same menu everywhere; C adds a second way."),
  ROW("import", "Open an existing LDraw file",
    "A banner says what was opened and offers Make it parametric; the Steps tab opens first.",
    "The same card docks at the right, Steps open on the left, \"Cottage (imported)\" in the project chip.",
    "A banner under the bar; the Parameters tab explains there's no model.py yet.",
    "All three make \"Make it parametric\" the one next step."),
  ROW("untrusted", "Untrusted project, offline, an old library",
    "Banners under the title push the layout down, never over the viewer; untrusted also locks the sliders with a reason.",
    "Banners dock as the bottom-right card; the capsule is warn-colored (\"Read-only until you trust it\").",
    "Banners dock under the bar; untrusted locks the sliders; offline disables Send.",
    "A's top strip is the most prominent; B keeps the model cleanest; C's docks can stack.", ["offline", "old-library"]),
];

window.REVIEW_RECOMMENDATION = `<h2>Recommendation: B · Canvas, with two things from A and C</h2>
<p><b>Why B.</b> It's the only direction where the model is plainly the hero on every screen, and its top-center capsule answers "what is the agent doing right now?" faster than either alternative: the eye lands there first, and it can't scroll away. Its warmth suits the people Emmet is for: brick builders who have never opened a terminal (D-41). The chat stays one line until you want it, which is the strongest answer to "less code in the chat" (note 2), and nothing covers the house in any of the 28 screens.</p>
<p><b>The mix I'd propose</b> (the fourth option in the question):</p>
<ul>
<li><b>B's canvas and capsule</b> as the base: the full-bleed viewer, the floating left card, the composer at the bottom, cards in the bottom-right corner.</li>
<li><b>From A, a "Chat focus" layout preset</b>: the history docked as A's full-height right column, pushing the viewer rather than sliding over it, for long sessions; toggled with ⌘L, the way ClawdCAD names its presets. B's sheet becomes that column.</li>
<li><b>From C, the keyboard</b>: the ⌘K command palette (not as the composer), numbered keys on question and approval cards (already in all three), and the step name on the scrubber ("Step 8/12 · Ceiling").</li>
</ul>
<p><b>What B has to prove in 2B:</b> the floating panels at the smallest window (1024 × 700), a two-column left card when the window is wide, and the question and approval cards' link to the composer (a connector or a shared edge).</p>`;

window.REVIEW_CONTRAST = `<p>Two checks, both in the PR:</p>
<ul>
<li><code>node scripts/contrast-check.mjs</code>: every text token against every surface (4.5:1), every icon and control edge (3:1), and every filled control against its text, in both themes of each direction, the brand palette, the harness, and this page; and no color anywhere outside the tokens. A: 101 + 101 pairs, B: 91 + 91, C: 133 + 133, brand: 41 + 41, all pass.</li>
<li>Each prototype's <code>#audit</code> (for example <code>directions/b/index.html#audit</code>, which also runs in headless Chrome from <code>file://</code>): it renders all 28 screens in both themes and checks every visible text element against what is really behind it, failing text that sits on the render with no surface of its own. A: 4,368 elements, B: 2,458, C: 3,258; 0 failures.</li>
</ul>`;
