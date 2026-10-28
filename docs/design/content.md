# Emmet's words: content and tone

Final (plan session 2B). Every word in the app, the guide, and the site follows this; the frozen screens are the
examples. Section 14's rules, made specific.

## Voice

Emmet talks like a friend who builds: calm, direct, and specific. It says what happened, in numbers when it can, and
what to do next. It doesn't cheer, apologize, or explain itself at length.

- **Plain words.** "3 parts aren't connected to anything", not "floating component count 3". "The model didn't
  build", not "MODEL_EXEC_FAILED".
- **Numbers over impressions.** "Built 163 parts in 0.1 s", not "Built quickly". "96 of 150 MB", not "Almost there".
- **Names, not subtext** (Barry 2026-10-27). A control has a short name; what it means is its tooltip. No help line
  under every option.
- **Say each thing once** (Barry 2026-10-27). If a card says the agent is waiting on you, the status doesn't say it
  too, and nothing says "Idle".
- **No code in the app** (law 2). No Python, no file names or line numbers, no tracebacks, no diffs, no error codes or
  finding ids. They go to the agent with "Ask agent to fix", and to "Copy the details".
- **Teach one thing at a time,** at the moment it's useful: the first knob, the first finding, the first export.

## Mechanics

- **US English:** color, gray, center, labeled, behavior, canceled.
- **Sentence case** for everything: buttons, tabs, titles, settings, menu items on Windows ("Save version", "Ask agent
  to fix"). macOS menu bar items take title case, as macOS does ("Save Version").
- **Buttons are verbs** for what happens: "Save version", "Trust this project", "Try again", "Continue with Codex".
  The primary button is the thing to do next; there's one per surface.
- **Contractions** are fine and friendly: "isn't", "didn't", "you're".
- **Numbers:** digits for every count ("3 parts", "1 problem"); thousands with a comma (1,000); ranges with an en dash
  (50–250); a plus for open ranges (2,500+); percent with no space (16%, 20%); units with a space (150 MB, 6 studs).
  Dimensions with a times sign (32 × 32, 10 × 8 studs); part names stay as LDraw writes them ("Brick 1 x 2").
- **Time:** an elapsed time as m:ss (0:22); a time of day as the system writes it ("Today 14:31"); a wait as hours and
  minutes ("resets in 2h 14m"); "Updated 8m ago".
- **Agents by name:** Claude and Codex in the chat; Claude Code and Codex where it's the program (install, restart,
  version). Their plans as the provider names them: "Max plan", "ChatGPT plan".
- **Our name:** "Emmet Builder" where people first meet it (first run's wordmark, About, the installer); "Emmet"
  everywhere else.
- **Brands (D-30):** never LEGO, minifig, or Miniland as names in the UI; plain words ("figure scale", "Large"). The
  trademark line appears in About and on the site's footer, as TRADEMARKS.md words it.
- **Ellipses** only on a menu item or button that asks for more before it acts ("Open…", "Export…"), and in a
  placeholder that's a prompt ("Redirect Claude…").
- **Quotes:** curly quotes around a file's name ("Saved “Cottage parts.csv” to Downloads").

## Patterns

| What | Pattern | Example |
|------|---------|---------|
| A problem | What's wrong, in a plain sentence, then the one action | "9 parts aren't held by anything. The chimney starts 9 plates above the baseplate, so nothing holds it up." · Ask agent to fix |
| An error | What happened · what's safe · what to do | "The model didn't build. The agent's last change has a mistake in it, so the new model couldn't be built. The last good model is still shown." · Ask agent to fix |
| A crash | What stopped · that your work is safe · one fix | "Claude Code stopped unexpectedly. It stopped in the middle of a turn. Your project is untouched: the model and the last build are as they were." · Restart Claude Code · Copy the details |
| Something unavailable | Why, then what still works | "You're offline. Building, checks, knobs, and exports still work. The agent and downloads need a connection." |
| A wait | What, how much, and that it's safe to move on | "Downloading the LDraw parts library (about 150 MB) … This takes a minute or two, and only happens once." |
| A failure with a retry | What stopped, where it picks up, Try again | "The download stopped. The connection dropped at 96 of 150 MB. Emmet picks up where it stopped." · Try again |
| A confirmation (toast) | What was done and where, then the next thing | "Saved “Cottage parts.csv” to Downloads" · Show in folder |
| The agent's status | What it's doing now, in the present, with the time | "Building the walls, row 3 of 6 · 0:22", "Fixing the floating chimney", "Waiting for your answer" |
| A tool card | What was done, in the past, then the detail | "Built 70 parts · Walls, rows 1–3, and the room inside", "Checked: no problems" |
| A change to the model | What changed, in words | "Moved the chimney down · It starts on the baseplate now" |
| A question from the agent | The question, the answers recommended first, then Other | "Which style should the cottage have?" |
| An approval | What, exactly, and why, then the three choices | "Open a web page? example.com/stone-cottage.jpg · You pasted this link…" · Allow once · Allow for this session · Don't allow |
| A risk | Say it once, plainly, where the choice is | "A project can run code on your computer. Trust it only if you know where it came from." |
| An empty state | What goes here and how it starts | "An empty baseplate and your agent" |
| A setting's tooltip | What it decides, in one line | Scale: "What the details are sized for; the whole can be shortened to stay buildable" |

## Words we use

| Use | Not |
|-----|-----|
| part | piece (except Pieces, the build setting that counts them), brick (when it isn't one), element |
| model | design, creation, build (as a noun for the model) |
| build (verb), the build (a run) | compile, render (for a run) |
| knob, Tweaks | parameter, variable, slider (in the UI) |
| build settings | prompt settings, config |
| step | instruction, stage (except the agent building "in stages") |
| sub-assembly | submodel (in the UI; the DSL and LDraw say submodel) |
| problem, finding (in Checks) | error (unless the build failed), warning, issue |
| version | snapshot, commit, checkpoint |
| the agent, Claude, Codex | the AI, the bot, the assistant |
| sign in | log in, login |
| trust this project | allow, unblock |
