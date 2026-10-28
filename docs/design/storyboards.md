# Information architecture and storyboards (2A)

The nine key flows from plan section 14 as short annotated storyboards. Each frame names its screen in
`shared/states.js`; open it in any direction with `directions/<a|b|c>/index.html#s=<screen>&t=light` (or `t=dark`),
from the folder, no server needed. The words are the app's real ones (from 1C-1F and the transcripts); the numbers are
the cottage template's (163 parts, 31 steps; knobs for weathering, the roof, the chimney, and Shuffle).

## Information architecture

```
Emmet window (one project per window)
├── Title: project name and menu · build state ("Built 163 parts in 0.1 s · no problems")
├── Viewer (the hero, law 1): the model · views (Iso, Front, Top) · frame · screenshot · step scrubber
│   └── on the model: a finding's pin and highlight · the build animation and Skip
├── The agent
│   ├── Now: what it's doing, its time, Stop (Esc)              ← note 1: always where the eye is
│   ├── Conversation: messages · tool cards (one plain line, raw folded) · file changes (diff folded)
│   │   · question cards · approval cards · pending redirect · errors with "Ask agent to fix"
│   ├── Header: agent picker (Claude Code, Codex) · which plan pays · Start fresh   ← note 3
│   └── Composer: message · attach a photo · model and effort · Send / Stop        ← note 4
├── The model's panels
│   ├── Parameters: Size · Colors · Options, in words, help from the docstring; Keep as default / Reset  ← note 5
│   ├── Checks: findings, each with its parts, its model.py line, and "Ask agent to fix"
│   ├── Parts: the BOM (part, color, quantity) · Export
│   └── Steps: the list; a step shows the model up to it                         ← note 6
├── Status: agent state · LDraw library and parts data versions · plan usage meters
└── Around the project: first run · New (the Cottage example or a prompt) · Open (a folder or an LDraw file)
    · Settings (default folder, agents, keys) · About (the trademark line)
```

Banners sit under the title, above the viewer, and never cover it: untrusted (read-only until Trust), offline, an old
library, an imported file with no model.py.

## 1. First run (D-41: no terminal)

| # | Screen | What the user sees | Why |
|---|---|---|---|
| 1 | `fr-welcome` | "Build brick models by describing them", three plain promises, Get started | Says what Emmet is before asking for anything |
| 2 | `fr-library` | The LDraw library (about 150 MB) and Python downloading, with real progress | The one slow step, honest about time; it keeps going while the user moves on |
| 3 | `fr-agents` | Claude Code: ready, Max plan. Codex: "Install Codex". Which plans work. The provider notice. "Check agents again" (note 10) | Install and sign-in by click (1F); the user learns which plan pays |
| 4 | `fr-start` | "Start with the Cottage example" or "New model from a prompt"; projects go to Documents/Emmet (notes 7, 9) | A good first model in one click, or straight to the prompt |

## 2. New model from a prompt

| # | Screen | What happens | Agent status says |
|---|---|---|---|
| 1 | `new-empty` | An empty baseplate (note 8); the composer holds the rung 1 prompt | (idle) |
| 2 | `thinking` | The message is sent; a collapsed "Thinking" line | "Thinking about the cottage · 0:04" |
| 3 | `question` | The agent asks "Which style should the cottage have?" (from the rung 1 transcript); work waits | "Waiting for your answer" |
| 4 | `building` | Rules read, parts found, model.py written (folded); 52 parts drop in, Skip | "Building the walls, row 3 of 6 · 0:22" |
| 5 | `redirect` | 108 parts; the user typed "put the chimney at the back, like the stone one on this house:" and pasted a link: a pending bubble | "Building the walls and ceiling · 0:31" |
| 6 | `approval` | The redirect is delivered and acknowledged; the agent asks to open the pasted link (its own web tool, outside its sandbox, D-40) | "Waiting for you to allow a web request" |
| 7 | `built` | 163 parts, no problems: a real door, glass windows, a room inside; the finished run folds to one line; the agent's summary | "Done in 1:12" |

## 3. Tweak with sliders

| # | Screen | What happens |
|---|---|---|
| 1 | `built` | The model's knobs (Barry 2026-10-27: character, not raw dimensions): Weathering 0 %, Chimney, Variation 1 with Shuffle |
| 2 | `sliders` | Weathering dragged to 20 %: old Reddish Brown brick shows through the Tan walls in 0.1 s, the camera still; "was 0 %"; "1 change · not saved to model.py yet", Reset, Keep as default |
| 3 | `shuffled` | Shuffle: another random arrangement of the same weathering, at once and free |

## 4. Fix a finding

| # | Screen | What happens |
|---|---|---|
| 1 | `findings` | "9 parts aren't held by anything": the chimney is highlighted, the rest dimmed; the finding shows model.py lines 253-254 and the hint from `emmet check` |
| 2 | `fixing` | "Ask agent to fix" sent the finding (the form the fix-collision transcript records); the agent edits line 253 and rebuilds |
| 3 | `built` | 163 parts, 0 findings |

## 5. Export

| # | Screen | What happens |
|---|---|---|
| 1 | `built` | Parts: 163 parts in 29 kinds |
| 2 | `export` | Parts list: CSV, Rebrickable, BrickLink XML · Model: LDraw .mpd · Instructions: PDF, LPub3D-ready |

## 6. Open an existing LDraw file

| # | Screen | What happens |
|---|---|---|
| 1 | `import` | `cottage.mpd` opens as an imported project: 163 parts in 31 steps, the steps list, no sliders yet; "Make it parametric" asks the agent |

## 7. Switch agent mid-project

| # | Screen | What happens |
|---|---|---|
| 1 | `switch-agent` | The picker: Claude (Max plan), Codex (ChatGPT plan); "Switching starts a fresh conversation. This one is kept." |
| 2 | `switched` | A divider, then Barry's 1C garage request goes to Codex, which reads the project files first |

## 8. Answer a question

| # | Screen | What happens |
|---|---|---|
| 1 | `question` | Two answers and Other; keys 1-3 pick; the status says it's waiting on you |
| 2 | `building` | The card collapses to the answer; the agent carries on in the same turn |

## 9. Watch, redirect, stop

| # | Screen | What happens |
|---|---|---|
| 1 | `building` | Parts drop in by step; Skip shows the finished stage at once |
| 2 | `redirect` | A pending bubble: Edit, Cancel, "Stop and send now"; it goes at the agent's next step |
| 3 | `approval` → `built` | Delivered and acknowledged; the agent asks to open the pasted link, then builds the chimney at the back |
| 4 | `stopped` | Or Esc: "You stopped the turn at 0:41. The model is as its last run left it: 108 parts." |

## The other states

| Screen | Section 14 state | What the user sees |
|---|---|---|
| `failed` | Build failed | "The model didn't build": the real TypeError, model.py line 121, Ask agent to fix; the last good model stays |
| `crashed` | Agent crashed | "Claude Code stopped unexpectedly"; the project is untouched; Restart |
| `signed-out` | Agent not installed or not signed in | "Claude isn't signed in": Sign in to Claude, or Use Codex instead |
| `untrusted` | Untrusted project | Read-only, with the real message from `emmet run`; Read model.py, Trust this project |
| `offline` | Offline | Building, checks, sliders, and exports work; the agent and downloads wait |
| `old-library` | An old library version | 2026-06 against the expected 2026-09; Update the library |
| `model-picker`, `usage`, `steps` | (notes 4, 6; section 10's usage) | Model and effort; plan usage; step 9 shows only what's built by then: the stove, in while the walls are low |
| `step-door` | (plan section 08's step rules, Barry 2026-10-27) | Step 4 as the instruction books draw it: the parts box, the step number, the new parts outlined in red, and the door built apart in its own box with a line to where it goes |
