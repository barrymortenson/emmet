# Design and feature notes

Barry's notes from using Emmet, newest first. 2A reads this file before it starts, and each direction answers every
layout note. Each note says which session builds it. Add new notes at the top with the date.

## 2026-10-27, from building with official instruction books

Barry compared the cottage's steps (15, one per wall row, the room furnished after the walls were up, the chimney
in one stack at the end) with the official books he builds from. Homes proposed, not yet approved by Barry.

11. **Steps like an instruction book.** Build bottom up a layer at a time, or a section at a time (a wall at a time),
    with a side left open to put interior features in. Steps are small groups of parts. Adding a feature means
    redoing the step order, not a step at the end. Each step shows its parts the way the books do (a parts box with
    counts), and a unit built apart is a sub-assembly in its own box. This matters most for the PDF and on-screen
    instructions. Plan section 08 now has the step rules; 3B builds sub-assemblies and `steps.json`, 4C the steps
    view, 4E the PDF; the doctrine teaches it now, and the cottage template follows it (31 steps, the room furnished
    at the second row, the chimney going up with the walls). 2A's mock shows step 4 (the door as a sub-assembly) and
    step 9 (the stove) as the books would.

## 2026-10-25, from the 1C live check

Barry used 1C's plain chat pane on the cottage, with Claude Code on his Max plan and Codex on an API key.

| Emmet after 1C, with Claude | Emmet after 1C, with Codex |
|---|---|
| ![1C chat pane with Claude](notes/emmet-1c-claude.png) | ![1C chat pane with Codex](notes/emmet-1c-codex.png) |

### Layout and design (2A designs them; the session named builds them)

1. **Show what the agent is doing, in context.** The status about the agent working belongs above the chat, where
   the user is looking, with a spinner, so it's clear the agent is still working. The chat sometimes looked stalled.
   1C pinned an "agent is working" line with a spinner and a seconds counter above the composer. 2A decides its
   final place and look; 4B builds it.
2. **Less code in the chat.** The chat showed too much code. 4B's plain-language tool cards, with the raw input and
   output collapsed, are the answer; 2A should show how little code each direction needs by default.
3. **Agent switching and Start fresh, like nurb.** nurb's chat header has the agent picker and a Start fresh button.
   Switching agents starts a fresh conversation and keeps the old one, and Start fresh clears the context without
   switching. nurb's picker also offers Gemini, with a "sign in" link when it isn't set up (D-42). 4B builds it.

   ![nurb's agent picker and Start fresh](notes/nurb-agent-picker.png)

4. **A model and effort picker, like nurb.** nurb's composer has Model and Reasoning effort, notes that "Bigger
   models and higher effort use up your plan faster", and shows the choice next to Send. 4B builds it (D-41's note
   on small plans).

   ![nurb's model and effort picker](notes/nurb-model-picker.png)

5. **How useful are the sliders?** Barry questions the sliders as they stand (width, depth, wall rows, colors,
   chimney, garage). 2A looks at what nurb's sliders do well and proposes which parameters show, how they're grouped
   and labeled, and how they read in each direction. 3B owns which parameters the templates expose.
6. **A step shows the build up to that step.** Clicking a step should show the model built up to it and hide the
   bricks of later steps. Today clicking a step highlights it but shows the whole model. Section 13's step scrubber
   now says later steps are hidden, and 4C builds it, from the steps list as well as the scrubber.

### Features (homes approved by Barry, 2026-10-25)

7. **A default Emmet folder for new projects, which the user can change.** 4A: New saves to `Documents/Emmet` by
   default, and Settings changes the folder.
8. **The baseplate as an option.** A baseplate of about 30 × 30 studs by default, which the user can turn off. The
   cottage already stands on the standard 32 × 32 baseplate (part 3811), the closest real size. 3B: every template
   gets a baseplate on/off parameter; with it off, the model starts at the ground instead of on the plate.

## 2026-10-26, from the 1E hand run

Homes proposed, not yet approved by Barry.

9. **New can't start a blank model.** New always makes the cottage, so "New model from a prompt" (section 14's key
   flow) has no way in from the app; the CLI already has `emmet new --template empty`. Proposed home: 4A, with New
   offering "Start with the Cottage example" or "New model from a prompt" (an empty baseplate).
10. **"Look again" doesn't say what it does.** It runs agent detection again (each CLI's version and sign-in), for
    after installing an agent or signing in from a terminal. Proposed home: 4A, renamed (for example "Check agents
    again") or replaced by re-detecting when the window regains focus.
