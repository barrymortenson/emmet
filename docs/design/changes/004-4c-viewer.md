# 004: What session 4C's viewer adds to the frozen screens

Session 4C built the viewer's interactions to `docs/design/frozen/` and `components.md`'s viewer. Where the frozen
screens don't draw a state, or plan section 13 asks for something they leave out, the app does the following.

## Changes

| Where | Frozen | The app | Why |
|---|---|---|---|
| The view switch | Iso, Front, Top, Frame, Screenshot | The same, then Replay build (`play`) | Section 13 puts Replay build in the viewer's toolbar; it's disabled while Settings' build animation is Off |
| Screenshot | An icon button | A click copies a picture of what the viewer shows; the toast, "Copied a picture of the model", offers Save to file, then "Saved “Cottage.png” to Downloads" with Show in folder. The View menu has Copy Screenshot and Save Screenshot | Barry, 2026-10-31 (`docs/decisions.md`) |
| Hovering a part | Not drawn | The part's name and color in a tooltip beside the pointer ("Brick 2 x 4 · Red"); the part draws a little lighter | Section 14: "Hovering shows the part's name and color" |
| Clicking a part | Not drawn | It's picked, as a row in the panel is: the rest fade 80 %, and the agent sees it. Shift, ⌘, or Ctrl adds or takes away; a click on nothing clears | Section 13's pick; the highlight is components.md's |
| Shift-drag | Not drawn | A dashed accent outline while the box is drawn; the parts whose centers are in it are picked | Barry, 2026-10-31 |
| The cut-away | Not drawn | View menu Cut Away Above (`⌥↓`, `⌥↑`), and a bar over the viewer: "Showing layers 1–8 of 24", lower, raise, Show all | Barry, 2026-10-31 |
| A finding's pin | One pin, "9 parts float" | A pin for each finding with parts, at most five (errors first), just above its parts and following the camera; click opens Checks with it picked | A model with many findings points at the worst few; Checks lists them all |
| A step | The parts box, the number, the door's sub-assembly box with its leader | The same from the build's steps: the step's new parts outlined in red in the viewer, each new part's picture and count, and each sub-assembly it puts on with its parts and numbered sub-steps; the leader runs down and across to the first copy, or across from the box's right edge when the copy is higher | Section 08's step rules |
| Settings → Look | Viewer quality: Fast, Balanced, Best | The same, showing the one picked for this computer's graphics until the user picks; then Build animation: Animated, Fast, Off | Section 13's quality presets (auto by GPU) and the animation's pace |
| Highlights | A finding's parts tinted red; a pick in their own colors | The same, each with what isn't color: a finding has its pin and words, an agent's pointing its bar naming the agent, a pick the rest of the model faded back, a hovered part its name; a step's new parts an outline | Accessible, not by color alone |

## Barry's yes

Asked through AskUserQuestion when the screenshots were ready.
