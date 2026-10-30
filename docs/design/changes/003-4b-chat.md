# 003: What session 4B's chat adds to the frozen screens

Session 4B built the chat to `docs/design/frozen/`. Where the frozen screens are silent, or what they draw needs
something the agents don't give, the app does the following. The side-by-sides in
`docs/spec/sessions/4B/side-by-side/` show each frozen chat screen beside the app.

## Changes

| Where | Frozen | The app | Why |
|---|---|---|---|
| The status | The agent's own words ("Building the walls, row 3 of 6") | What the running step is, from the tool table ("Building", "Checking the model", "Looking for parts"), "Thinking", "Writing", or "Reading your message", with the time | Neither CLI says what it's about to do in a line of its own; the agent's words are the latest line under it |
| The composer | Attach a photo | Not shown | 4D builds photo to model |
| Send | Always the primary color | Grayed out while the message box is empty | Nothing to send yet |
| The model chip | "Opus 5.5 · Medium" | The model the CLI reports ("Haiku 5.5"), and the effort only when one is picked | Neither CLI says its default effort |
| Model and effort | Two selects and the note | The same, with each CLI's own list: Claude Code's documented aliases (Default, Fable, Opus, Sonnet, Haiku) and its five levels; Codex's `model/list` and each model's efforts | Only what each CLI documents (plan section 21) |
| The agent picker | Claude · Max plan, Codex · ChatGPT plan, the note | The same, an agent that isn't ready says "Not signed in" or "Not set up", and "Check agents again" | Note 10's re-detect, reachable from the dock |
| The plan badge | "Max plan · 16%" | The plan's name from detection until the first report (no number before the user acts, D-36); "API key" for Codex on a key | Usage is fetched only when the user acts |
| Plan usage | Meters, "Updated 8m ago", Codex's line | The same, with Refresh, usage credits when reported, and "This conversation: N tokens in, M out" where the CLI reports it | The session's usage display (4B's scope) |
| Not signed in | An alert card with Sign in and Use Codex instead | 1F's setup cards for each agent that isn't ready (Install, Sign in, which plans work), and "Use Codex instead" when Codex is ready | One copy of the install and sign-in flows |
| A redirect waiting | Edit, Cancel, Stop and send now | Edit and Cancel; Stop and send now after 20 seconds without delivery | Plan section 10 |
| An approval | The question, the tool, what, why | The same; the why is Emmet's words for the kind of request (the CLIs don't give a reason). Answered, it folds to a line ("You said no: open a web page. Claude carries on without it.") | |
| An approval or question card | Keys 1–3 and 1–5 | The card takes focus only when nothing else has it, and its keys work after a moment | A key meant for the message box can't answer it (4B's security review) |
| A message the app wrote | "Sent from Checks", "Sent from Build settings" | Also "Sent from the failed build" and "Sent from the imported file" | Ask agent to fix on a failed build, Make it parametric |
| A crash | Restart, Copy the details | The same; Copy the details copies the diagnostics | 6B's bundle comes later |
| The agent's replies | Paragraphs and bold | Paragraphs, lists, bold, italics, and inline code; a code block folds to "Code · N lines" | No code on screen unless asked (law 2) |
| Undo turn | The button | The button and ⌥⌘Z, then a toast: "Undid Claude's last turn. The model before is version 7." | Says where the replaced model went |
| Tips | Not drawn | One at a time, at the top of the tab where it's useful (Tweaks, Checks, Parts), with Got it; each once per user | Section 14: the first knob, the first finding, the first export |

## Barry's yes

Waiting for Barry's yes, through AskUserQuestion, after the side-by-sides.
