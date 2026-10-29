# 001: What session 4A's build adds to the frozen screens

Session 4A built the shell, first run, the panel, and Settings to `docs/design/frozen/`. Where the frozen screens are
silent, or a later session's work isn't in yet, the app does the following. The side-by-sides in
`docs/spec/sessions/4A/side-by-side/` show each frozen screen beside the app.

## Changes

| Where | Frozen | The app | Why |
|---|---|---|---|
| Settings → Agent | "Model and effort: Opus 5.5 · Medium" | "Claude's model": Claude Code's default, Opus, Sonnet, or Haiku | Effort and Codex's model come with 4B's model and effort picker beside Send; Claude Code takes a model now |
| Settings → Agent | "Codex runs on: Using ChatGPT plan · Manage usage" | 1E's account list, Continue with ChatGPT, and the API key field under "Codex runs on" | 1E's sign-in and key live in Settings (OpenAI's requirements); the frozen screen shows a signed-in plan user |
| Settings → Look | Color names: LDraw, BrickLink | LDraw, Rebrickable, BrickLink | 3B's three namings |
| Settings → Emmet | "Checked today · 0.1.0 is the latest" | "0.1.0 · Update checks start with the first release"; a Parts data row (3B's Settings → Data) | Updates are 6A's |
| About | Licenses button | Licenses shows the attributions: LDraw.org and the part authors, Rebrickable, the LDCad Shadow Library (D-44), Lucide, the provider notice (D-37), Emmet's licenses | Not drawn; 4A's scope lists them |
| ⌘/ | Not drawn | A dialog listing keyboard.md's shortcuts, from the dialog, data-table, and kbd components | keyboard.md names it |
| First run, Start | "Or open a folder, or an LDraw file …" as text | The words open Open… | A way in from that screen |
| First run, Start | "163 parts: a real door, …" | "147 parts: …" | The Cottage template builds 147 parts (3C rebuilt it in bond); the first-run E2E checks the card against the build |
| First run, Your agent | Continue with ChatGPT and Use an API key on Codex's card | Both open Settings, where 1E's sign-in runs | One copy of OpenAI's flow |
| The dock | 4B's chat | 1C's chat pane, in the dock, in the tokens; the failed-build card above it | 4B rebuilds the chat |
| Build failed | "The model didn't build" | The same for a model's error; "The build stopped", Emmet's words, and Rebuild when Emmet itself failed (not the model) | The frozen words blame the agent's change |
| The title row | Screenshot | Shown, not yet enabled | 4C's |
| Export | PDF | Shown, not yet enabled | 4E's |
| First run | The card over the model, with no title row | The top edge and the model behind the card move the window, as the title row does | First run covers the title row |
| Launch | Not drawn | Reopens the last project; with none, first run's Start | |

## Barry's yes

Yes, as written: Barry, through AskUserQuestion, 2026-10-29, after the side-by-sides and the part pictures' fix
(no background, as the frozen Parts list draws them).
