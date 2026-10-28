# Emmet's components

The frozen design system (plan session 2B): direction B, Canvas, as Barry picked it (H-07). Phase 4 assembles the
app from these components and nothing else. Each one is drawn in every state in
[`frozen/components.html`](frozen/components.html), styled in [`frozen/components.css`](frozen/components.css) (one
section per component, `/* @component <id> */`), and built in [`frozen/components.js`](frozen/components.js); every
frozen screen (open it from [`frozen/index.html`](frozen/index.html)) lists the components and tokens it uses in its Uses panel.

## How to read this

- **Tokens only.** Every color, size, radius, shadow, and duration is a token in
  [`apps/desktop/src/styles/tokens.css`](../../apps/desktop/src/styles/tokens.css). `components.css` has no colors
  and no pixel sizes (`tests/scripts/design.test.mjs` checks), so 4A's lint already holds for it.
- **Radix underneath (D-29).** Where a component says "Radix: Tabs", 4A uses that primitive for its behavior
  (focus, keys, ARIA) and puts these classes on its parts. The frozen markup already has the roles and `data-state`
  values the primitive sets (`data-state="active"`, `"checked"`, `"open"`, `"on"`), so the CSS carries over as is.
  "Own" means no Radix primitive exists for it; the entry says how it behaves.
- **States.** Hover, press, and focus come from `:hover`, `:active`, and `:focus-visible`. `data-show` draws them
  still on the specimens page only; 4A leaves it out. `data-c="<id>"` marks each component's root for the Uses lists
  and the audit; 4A may keep it or not.
- **Focus.** Every focusable thing shows the focus ring: `--focus-width` in `--focus-color`, `--focus-offset` away
  (the accent on macOS; the text color, closer, on Windows).
- **Labels.** Every control has a name a screen reader reads: its text, or `aria-label` (icon-only buttons use their
  tooltip text). The viewer's text alternative is the Steps tree and the Parts list (section 14).
- **Words.** Labels and messages follow [`content.md`](content.md); keys follow [`keyboard.md`](keyboard.md); icons
  follow [`icons.md`](icons.md).

## Tokens

| Kind | Tokens | Notes |
|------|--------|-------|
| Color | `--bg-canvas` `--bg-surface` `--bg-raised` `--bg-sunken` `--bg-accent` `--bg-bad` · `--fg-text` `--fg-muted` `--fg-accent` `--fg-ok` `--fg-warn` `--fg-bad` · `--ui-control` `--fill-accent` `--on-accent` | 15 per theme. Every `--fg-*` is 4.5:1 on every `--bg-*`; `--ui-control` and `--fill-accent` 3:1; `--on-accent` 4.5:1 on `--fill-accent` (`scripts/contrast-check.mjs`). `--bg-canvas` is the viewer's background, the one the renders are made on. |
| Mixed | `--line` `--scrim` `--elevation-1` `--elevation-2` `--elevation-3` | Mixed from the colors above: the hairline from the text color; in light the scrim and shadows from the text color, in dark from the sunken surface. |
| Type | `--font-ui` `--font-display` `--font-mono` · `--text-2xs` 11 · `--text-xs` 12 · `--text-sm` 13 · `--text-md` 14 · `--text-lg` 16 · `--text-xl` 20 · `--text-2xl` 28 · `--text-step` 44 · `--leading` `--leading-tight` · `--weight-light` `--weight-regular` `--weight-strong` `--weight-bold` | System fonts: SF Pro on macOS, Segoe UI Variable on Windows. 14 px body. |
| Space | `--space-3xs` 2 · `--space-2xs` 4 · `--space-xs` 6 · `--space-sm` 8 · `--space-md` 12 · `--space-lg` 16 · `--space-xl` 24 · `--space-2xl` 32 · `--space-3xl` 48 | A 4 px rhythm. |
| Radius | `--radius-xs` 4 · `--radius-sm` 6 · `--radius-md` 10 · `--radius-lg` 14 · `--radius-xl` 20 · `--radius-full` | Soft and roomy; pills for what floats on the viewer. |
| Elevation | `--elevation-1` (a selected tab, a thumb) · `--elevation-2` (what floats on the viewer: the panel, the dock, the project chip, the view switch, banners, pins) · `--elevation-3` (menus, popovers, dialogs, toasts) | |
| Motion | `--duration-fast` 120 ms · `--duration-base` 180 ms · `--duration-slow` 260 ms · `--duration-spin` · `--duration-part` · `--ease-standard` `--ease-exit` · `--rise` 4 px | Subtle: color changes and a 4 px rise, nothing longer than 260 ms. Reduced motion sets every duration to 0 and `--rise` to 0; the spinner stops (a dotted ring), and the viewer skips the build animation. |
| Size and layout | `--control-sm` `--control-md` `--control-lg` · `--icon-sm` `--icon-md` `--icon-lg` `--icon-xl` · `--line-width` `--line-width-strong` · `--focus-*` · `--titlebar-height` `--titlebar-inset-start` `--caption-button-width` · `--panel-width` (384) `--panel-min` `--panel-max` · `--dock-history` `--dock-active-max` · and the fixed sizes of menus, dialogs, toasts, tooltips, thumbnails, and pictures | |
| Platform | `[data-platform="macos"]`, `[data-platform="windows"]` | macOS: SF Pro, a 56 px title row with room for the traffic lights, the accent focus ring. Windows: Segoe UI Variable, a 48 px title row across the window with the menu bar and the caption buttons, the focus ring in the text color. |

The app sets `data-theme="light"` or `"dark"` on `<html>` from `prefers-color-scheme` (Settings can pin it) and
`data-platform` from the OS. The site (7A) imports `tokens.css` as it is.

## Foundations

### `base`
The window's type, color, and focus ring, which everything inherits; plus a few utilities (`.muted`, `.small`, the
tones `.ok` `.warn` `.bad` `.accent`, `.row`, `.grow`, `.num`, `.visually-hidden`). Not a component on its own.

### `icon`
A lucide icon or an Emmet glyph, 24-unit grid, stroke 2, in the text color. Sizes: `sm` 13, default 16, `lg` 20,
`xl` 24. Always `aria-hidden`; the control around it carries the name. The set is in [`icons.md`](icons.md).

### `spinner`
Work in progress: a ring turning once per `--duration-spin`, in the color of what holds it. Role `status` with a
label ("Building"). Reduced motion: still, drawn dotted.

### `kbd`
A key, beside the action it triggers (Stop `Esc`, a question's `1`–`5`). Uses the platform's names
([`keyboard.md`](keyboard.md)).

## Controls

### `button`
Radix: none (a `button`; `Slot` when it's a link). Variants: **primary** (filled accent; one per surface, the thing
to do next), **secondary** (outlined), **quiet** (no border, for the less likely choice). Sizes `sm`, default, `lg`
(first run). May lead with an icon and end with a key. States: rest, hover, press, focus, disabled (sunken, muted
text), busy (a spinner in place of the icon, the label says what's happening: "Installing"). Label: a verb, sentence
case ("Ask agent to fix", "Save version").

### `icon-button`
Radix: none, plus a Tooltip with its label. A round icon-only button (attach a photo, frame, screenshot, dismiss).
States: rest, hover, press, on (accent, for a toggle that's on), focus, disabled. Its `aria-label` and tooltip are the
same words.

### `badge`
A short fact in a pill: the plan ("Max plan · 16%", a button that opens usage), a tab's count, a tag ("Floating"),
"was 0%". Tones: default, accent (a changed value), warn (a count of problems; the plan from 90%), count (small
numbers).

### `text-field`
Radix: none (an `input`). One line: a version's name, Other on a question, the user's own style words. States:
empty (placeholder in `--fg-muted`), filled, focus, invalid (`aria-invalid` and a `--fg-bad` edge; the field's message goes under it, in `--fg-bad`), disabled.

### `slider`
Radix: Slider (Root, Track, Range, Thumb). A knob with a range (weathering 0–30%): the track, the filled range, the
thumb, and the value with its unit beside it. States: rest, hover (the thumb grows a little), dragging, focus,
disabled. Keys: arrows by one step, Page Up and Down by ten, Home and End. Moving it is a draft until Keep (the
setting shows "was 0%" and the panel a Keep bar).

### `switch`
Radix: Switch. The plan's "toggle": a knob that turns a feature on or off (Chimney, Roof). `data-state` checked or
unchecked. States: off, on, hover, focus, disabled. Space toggles it.

### `segmented`
Radix: ToggleGroup, type single. A few short choices side by side (Base, Age, Detail; Settings' Appearance). The
picked one lifts onto the surface. Arrows move between choices.

### `chips`
Radix: ToggleGroup. Choices that wrap (Scale, Pieces, Style). The picked one is accent-filled with an accent edge;
each chip's longer meaning is its tooltip. Style adds the user's own words as chips with a remove button, from the
text field under them.

### `select`
Radix: Select. A trigger showing the choice and a list of choices (the menu's look, role `listbox`): model, effort,
Settings' choices. States: closed, hover, open (accent edge), focus, disabled. Its label sits above it, or is
visually hidden where the row already names it.

### `color-picker`
Radix: Popover around a RadioGroup. A color knob: the trigger shows the swatch and the color's name; open, a grid of
swatches holding **only the colors the part is made in** (3B's parts data), each named by its tooltip, with "Only
colors this part is made in." The cottage has no color knob (colors go through the prompt, Barry 2026-10-27), so no
frozen screen shows one; a model whose agent makes a color knob uses this.

### `tabs`
Radix: Tabs. The panel's four views (Options, Checks, Parts, Steps), each with its count; Checks' count is warn.
`data-state` active or inactive. Arrows move between tabs; ⌘1–⌘4 jump to one ([`keyboard.md`](keyboard.md)).

### `tooltip`
Radix: Tooltip. A name's longer meaning, shown after a pause on hover or keyboard focus, gone on Esc: the build
settings' meanings, an icon button's label, a chip's description. Inverted (the text color as its background), small,
one or two lines. Never the only place something essential is said.

### `menu`
Radix: DropdownMenu. Actions with an icon, a label, and their key (the project menu), choices with a check (the agent
picker), group labels (Export's Parts list, Model, Instructions), separators, a note at the end, disabled items.
Arrows move, Enter picks, Esc closes, typing jumps.

### `menubar`
Radix: Menubar. Windows only, in the title row: File, Edit, View, Agent, Help, holding the same menus macOS shows in
its menu bar ([`keyboard.md`](keyboard.md)). Alt focuses it.

### `popover`
Radix: Popover. A small panel anchored to what opened it: plan usage, model and effort, Save version, the versions
list. Esc or a click outside closes it; focus goes back to its trigger.

### `dialog`
Radix: Dialog. Modal, over the scrim, focus kept inside, Esc closes: About and Settings. A title, the body, and the
actions at the end, primary last.

### `toast`
Radix: Toast. A short confirmation at the bottom center of the viewer, for five seconds: what was done and where
("Saved “Cottage parts.csv” to Downloads"), the one thing you'd do next ("Show in folder"), and dismiss. Never an
error (errors stay until dealt with, in a card).

### `progress`
Radix: Progress. Something that takes a while: what, how much in words ("96 of 150 MB"), and the bar. States:
loading, done, failed (the bar in `--fg-bad`, with an alert card and Try again).

### `meter`
A plan's usage (role `meter`): the share used, with the percent and when it resets beside it. Warn from 90%.

### `resizable`
Own (role `separator`, focusable, with its value). Between the viewer and the options panel (320–520 px, 384 by
default) and on the chat history's top edge (20–80% of the column). Invisible until hover or focus, then a grip in
`--ui-control`; arrows move it 16 px, Home and End to its limits. Both remember their size per window.

### `tree`
Own (the WAI-ARIA tree pattern, role `tree`). The Steps tab is the model's tree (Barry 2026-10-27, 2B): **Built
apart** first, each sub-assembly with how many times it's used ("× 5"), its part count, and its own sub-steps
(expandable); then the steps, grouped by their titles' first words ("Walls, row 2"). Clicking a step shows the build up
to it (later steps hidden, "Later steps are hidden"); clicking a sub-assembly highlights every copy in the viewer.
States: past, current, later (muted), selected, hover, focus, collapsed, expanded. Keys: up and down move, right and
left open and close, Enter picks, `[` and `]` step.

### `data-table`
Own (a `table`). The Parts list: the part's picture in its color, its name with the color and part number under it,
and the count, sorted by count. Rows: hover, selected (its parts highlighted in the viewer). The Rebrickable credit
under it (section 12). Its header sorts on click.

## The chat

### `chat-message`
The user's message (an accent bubble, right), the agent's text (plain, left, markdown without raw HTML), thinking
(collapsed, "Thinking", opens on click), a note (centered, muted: "You stopped the turn at 0:41"), a divider (an agent
switch), and a pending redirect (a dashed bubble: what you sent while the agent works, "Goes to Claude at its next
step", with Edit, Cancel, and "Stop and send now"). A message the app wrote for you says where from ("Sent from
Checks", "Sent from Build settings"); one delivered mid-turn says so.

### `tool-card`
Radix: Collapsible. A tool call in plain words: an icon for its kind, a title ("Built 70 parts"), a detail ("Walls,
rows 1–3"), and its end state: running (accent, a spinner), done (a check), failed (`--bg-bad`, an x). Open, it shows
the raw call, the only place a tool's own words show. A change to the model is a card in words ("Moved the chimney
down"), never a diff or code (law 2). Three or more finished cards fold into one line ("7 steps · built in 2 stages")
once the turn moves past them.

### `question-card`
The agent asks: a tag, the question, 2–4 answers (recommended first, with its description) side by side, then Other
with a text field. Keys `1`–`5` pick while it has focus. Answered, it folds to one line: the question, muted, and the
answer.

### `approval-card`
The agent wants to do something its sandbox can't contain (D-40): a tag (the tool), the question ("Open a web
page?"), exactly what (in mono), why, and Allow once, Allow for this session, Don't allow (keys `1`–`3`).

### `alert-card`
A card in the chat that needs you: the build failed ("The model didn't build", in plain words, Ask agent to fix, "The
last good model is still shown"), the agent crashed (Restart, Copy the details), not signed in (Sign in, Use Codex
instead, which plans work), the plan's limit (Continue with Codex, Wait for the reset), and first run's failed
download (Try again). Tone bad (`--bg-bad`) for something broken; default otherwise. Role `alert` when bad.

### `finding-row`
A check's finding in Checks (law 2: one click from its parts): the finding's glyph, what's wrong in a plain sentence
("9 parts aren't held by anything"), a tag (Floating), the why ("The chimney starts 9 plates above the baseplate, so
nothing holds it up"), and Ask agent to fix with It's fine. Clicking it selects it (an edge in `--fg-bad`) and
highlights its parts; its pin shows on the viewer. While the agent fixes it, the actions give way to "Claude is fixing
this". Allowed ("It's fine"), it goes quiet with Check it again. With no findings, Checks says "No problems" and what
was checked. The source line and hint go to the agent, never to the screen.

### `status`
What the agent is doing right now (Barry's note 1), at the head of the chat: a spinner or an icon, one line, the
time, and Stop with `Esc` while it works. Tones: busy (accent), attention (an accent edge: a question or an approval
waits on you, the plan's limit), bad (crashed), muted (stopped), done. Shown only when it says something the screen
doesn't (never "Idle", and not when the card under it says it bigger). Role `status`, polite.

### `composer`
The message box. It stays open while the agent works ("Redirect Claude…"), and what you send waits as a pending
redirect. Attach a photo; the model and effort chip (opens its popover); Send (`⌘↩`). Offline, Send is disabled and
the placeholder says why.

## The viewer

### `parts-box`
A step as instruction books draw it (section 08): the parts box (each new part's picture and count), the step number
under it (44 px, light), and a sub-assembly in its own box with its parts and numbered sub-steps, with a leader line
to where it goes (4C draws it from the part's screen position).

### `pin`
A finding's marker on the viewer, beside its parts: the floating glyph and "9 parts float". Clicking it opens the
finding in Checks.

### `view-switch`
Radix: ToggleGroup, then two icon buttons. Iso, Front, Top (keys `1`–`3`; `4`–`6` for back, left, and right from the
keyboard), then Frame (`F`) and Screenshot. Centered in the title row.

## The project

### `setup-card`
An agent in first run (and Settings): installed or not, signed in, which plan, and the one button that fixes it
(Install, Sign in), or Ready. Installing shows a busy button. Codex's sign-in is OpenAI's Continue with ChatGPT button
and "Use an API key"; the action wraps under the text when it's wide.

### `version-row`
A saved version in the versions list: its render, its name, when, the part count, and "saved by Emmet" for the
automatic ones (muted names). Clicking one opens it read-only in the viewer, with Restore, Open as new project, and
Back.

### `banner`
Across the top of the viewer, a line and its action: read-only (Trust this project), offline (Try again), an old
library (Later, Update the library, with the size), an imported file (Make it parametric). The icon carries the tone.

### `caption-buttons`
Windows only: minimize, maximize, and close at the title row's right end (Tauri's own window controls); close turns
`--bg-bad` on hover.

## Kept as the provider requires

Not Emmet's components, and not restyled: what OpenAI's UI/UX guidelines fix for Codex on a ChatGPT plan, as 1E built
it (`apps/desktop/src/ChatGpt.tsx`). The frozen screens show the first four in place.

- **Continue with ChatGPT**, in OpenAI's approved black format with the white ChatGPT logo (the only place the logo
  appears): first run's Codex card, and Settings' sign-in as 1E built it. Drawn from
  [`frozen/approved.css`](frozen/approved.css), 1E's style.
- **"Using ChatGPT plan · Manage usage"** near the composer whenever Codex runs on the plan, in a `badge` and a quiet
  `button`; Manage usage opens ChatGPT's usage settings in the browser.
- **The usage-limit card**: "ChatGPT · Usage limit reached", "Review your plan or this app's limit in ChatGPT
  settings.", and Manage usage as the primary action; no reset time, since OpenAI's error doesn't give one.
- **The one-time notice**, "You're using your ChatGPT plan", with Got it.
- **The invitation** to API-key users in Settings, "Use your ChatGPT plan in this app." (1E's; the frozen Settings
  shows a plan user, so it isn't drawn).

## Regions

The window's regions are composed from the components above; each screen shows them.

### `app`
The window: the viewer column on the left, a `resizable` separator, and the options panel on the right
(`--panel-width`). On Windows the title row runs across the whole window above both.

### `titlebar`
The title row, a drag region: on macOS after the traffic lights, the project chip (the ant, the project's name, and
its menu: New, Open, Save version, Versions, Export, About); on Windows the menu bar after it and the caption buttons
at the end. The view switch is centered over the viewer.

### `viewer`
The 3D view, the hero (law 1): the render at the camera's field of view, so a shorter viewer shows the whole model
smaller rather than cropping it. It carries only what points at the model: a finding's pin, a step's parts box, an
open version's bar (Restore, Open as new project, Back), Skip animation, and toasts.

### `dock`
The chat along the bottom of the viewer: the agent picker, the plan badge, the status, Undo turn (after a turn that
changed the model), History, and Start fresh; then the card that needs you or the latest line; then the composer.
History opens upward to `--dock-history` of the column and pushes the viewer up; it never covers the model.

### `panel`
The options panel: the model's state in a word with its icon (No problems, 1 problem, Building, Build failed,
Read-only, Empty) and Export, the tabs, and the tab's content.

### `setting`
One build setting or knob in the panel: its name (its meaning is the name's tooltip), "was …" when changed, an
accent edge while changed, then its control. Build settings (Base, Scale, Age, Detail, Pieces, Style) fold to one
summary line with Change once the model is built; a change waits in a bar with Undo and Rebuild (an agent turn).
Knobs (Tweaks) change the model at once and wait for Keep.

### `first-run`
First run: one card over the model, with the wordmark, a rail of its four steps (Welcome, Parts and Python, Your
agent, Start), and the step's content.

### `about`
About Emmet Builder, a dialog: the app icon, the wordmark, the versions, the license, what Emmet connects to, and the
trademark line (D-30).
