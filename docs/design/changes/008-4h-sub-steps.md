# 008: A sub-assembly's own steps in the viewer

The frozen Steps tree lists a sub-assembly's sub-steps under its row, but they're labels: only the row can be clicked,
and it lights every copy in the model. Barry, in his fifth e-bike session (2026-11-10): "the build apart sections do
not show the seperate steps, not sure how that is supposed to be used?" Session 4H makes a sub-step show its
sub-assembly on its own, as the PDF's framed boxes do.

## Changes

| Where | Frozen | The app | Why |
|---|---|---|---|
| A sub-step in the Steps tree | A label | A tree item: a click (or Enter) shows its sub-assembly on its own in the viewer, built up to it; it's the selected item, with the sub-steps after it muted like later steps, and its part count at the end of its row | Barry's ask |
| The Steps tab's heading | "Step 12 of 31", and "Later steps are hidden" | With a sub-assembly on its own: "Window: step 2 of 3", and "Built apart, shown on its own" | It says what the viewer shows |
| The viewer, with a sub-assembly on its own | (new) | The first copy alone, where it sits in the model, its sub-step's new parts outlined in red; a frame around the viewer, as the instructions draw around a sub-assembly; the sub-step's number and parts box at the top left (and the box of any sub-assembly it puts on, with its leader); no ground grid, pins, or cut-away meanwhile | The PDF's framed box, at the viewer's size |
| The viewer's bar | The cut-away's, an open version's, an agent's pointing | One more, like the cut-away's: the sub-assembly glyph, "Window (5x)", "2 of 3", the step before and the next (chevrons), and Back | A way on and a way out where the eye is |
| The camera | Moves only when asked (law 1) | Showing a sub-assembly frames it, from the side the camera is on, and stays put through its sub-steps; showing the model again puts the camera back where it was | The click asks for it: a window alone at the model's zoom is a speck. Law 1 holds: nothing moves the camera but the user's own action (the click, and the way out) |
| `[` and `]` | Previous and next step | With a sub-assembly on its own, its own previous and next step | "As a main step does" (the session's plan) |
| `Esc` | Close what's open, else stop the agent | A sub-assembly on its own counts as open: `Esc` shows the model again | The same rule |
| The sub-assembly's row | Lights every copy; again clears it | The same; from one of its sub-steps it shows the model again with every copy lit | One meaning for the row |

Icons: `chevron-left` and `chevron-right` (Lucide) join the set for the bar's step buttons (`icons.md`). No new tokens.
The frozen screens in `docs/design/frozen/` keep the signed design as it was drawn (its `components.css`, which the
app's is made from, gains the frame's rules, and `generated.js` the two tokens they use); this note, plan sections 13 and 14, and
`components.md` say what the app does.

## Barry's yes

Asked for (Barry, 2026-11-10, through AskUserQuestion: "Yes, later session"; docs/decisions.md). The screens are in
the session's PR for his read.
