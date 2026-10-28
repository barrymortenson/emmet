# Frozen design sign-off (UX-2, H-08)

Barry signs this to freeze Emmet's design (plan session 2B, D-16). Phase 4 builds to it exactly: 4A the shell, first
run, the panel, and Settings; 4B the chat; 4C the viewer. After it's signed, a change to any of it needs a design
change note (below) and Barry's yes.

## What's frozen

| What | Where |
|------|-------|
| Tokens: 15 colors a theme, type, space, radius, elevation, motion; light and dark; macOS and Windows | [`apps/desktop/src/styles/tokens.css`](../../apps/desktop/src/styles/tokens.css) |
| 41 components and 8 regions, each with its Radix primitive where one exists, and every state | [`components.md`](components.md), [`frozen/components.html`](frozen/components.html), [`frozen/components.css`](frozen/components.css) |
| 47 screens and states, at 1440 × 900 and 1280 × 800, light and dark, macOS and Windows, each listing the components and tokens it uses | [`frozen/index.html`](frozen/index.html) |
| The keyboard map and the menus | [`keyboard.md`](keyboard.md) |
| The content and tone guide | [`content.md`](content.md) |
| The icon set and the app icon | [`icons.md`](icons.md), [`icon/`](icon/), `scripts/app-icon.mjs` |

Open `frozen/index.html` from the folder (no server). Each screen's bar switches size, theme, and platform; clicks in
the window move through the flows.

## The screens and states

Section 14's states are in brackets.

**First run**
1. Welcome: what Emmet does, three things it gives you, Get started. [First run]
2. Parts and Python: the library download with its progress, Python next, Continue while it downloads. [Loading library]
3. The download failed: where it stopped, Try again. [Loading library]
4. Your agent: Claude Code ready, Codex with Install, which plans work, the provider notice. [Agent not installed or not signed in]
5. Sign in to Codex: OpenAI's Continue with ChatGPT button as 1E built it, or an API key. [Agent not installed or not signed in]
6. Start: the Cottage example or a new model from a prompt, and where projects go.

**Building**
7. New model from a prompt: an empty baseplate, every build setting open on Any, the prompt typed. [Empty]
8. Thinking: the status with Stop, the thought as one line. [Agent thinking]
9. A question: the card with two answers and Other; the status stays quiet. [Question waiting for an answer]
10. Building, brick by brick: the walls going up, Skip animation, Redirect in the composer. [Building, Build animating]
11. A redirect waiting: the dashed bubble with Edit, Cancel, Stop and send now. [Redirect pending]
12. An approval: open a web page the user pasted; Allow once, for this session, or not. [Waiting for approval]
13. Built and clean: the agent's last words, Done in 1:12 with Undo turn, build settings folded, the knobs.
14. The history opened: the whole conversation above the composer, the viewer pushed up, not covered.

**Knobs and settings**
15. A knob moved: weathering at 20%, "was 0%", Reset and Keep.
16. Shuffle: another arrangement, at once.
17. Build settings changed: style and detail, with Undo and Rebuild.
18. A setting's tooltip: what Scale decides.
19. The agent reworking the model for the new settings. [Agent thinking]

**Steps**
20. Step 4: the parts box, the step number, the door built apart in its own box with a line to where it goes.
21. Step 9: the room while the walls are low; later steps hidden.
22. A sub-assembly selected in the Steps tree: "Window × 5" (4C highlights every copy in the viewer and dims the rest).

**Checks and failures**
23. A finding: the pin on the chimney, the plain sentence, Ask agent to fix, It's fine. [Checks with findings]
24. The agent fixing it: the finding says so; the status names it. [Building]
25. The build failed: the card in plain words, Ask agent to fix, the last good model still shown. [Build failed]

**The agent**
26. Stopped: a quiet note and the model as its last run left it. [Stopped by the user]
27. Crashed: Restart Claude Code, Copy the details; the project untouched. [Agent crashed]
28. Not signed in: Sign in to Claude, Use Codex instead, which plans work. [Agent not installed or not signed in]
29. The agent picker: Claude or Codex; switching starts a fresh conversation.
30. Switched to Codex: the divider, the user's request, Codex reading the project; "Using ChatGPT plan · Manage usage" by the composer. [Agent thinking]
31. Model and effort, beside Send, with the note on plans.
32. Plan usage: Claude's three meters and when each resets; Codex's billing state with Manage usage.
33. ChatGPT's usage limit: OpenAI's card, Manage usage first, no reset time (1E).
34. The one-time "You're using your ChatGPT plan" notice, with Got it (1E).
35. Claude's plan limit reached: Continue with Codex or wait; the plan badge in warn.

**The project**
36. The project menu: New, Open, Save version, Versions, Export, About.
37. Save a version: a name and Save.
38. Versions: each with its render; Emmet's own muted.
39. An earlier version opened: Restore, Open as new project, Back.
40. Export: parts list (CSV, Rebrickable, BrickLink), model (LDraw), instructions (PDF, LPub3D).
41. Exported: a toast with Show in folder.
42. An LDraw file opened: the banner with Make it parametric; the Steps tree.
43. About Emmet Builder: the icon, versions, what Emmet connects to, the trademark line.
44. Settings: agent, Codex's billing, projects folder, appearance, color names, viewer quality, updates, the command line tool, diagnostics.

**Banners**
45. Read-only until trusted: Trust this project; the knobs locked. [Untrusted project]
46. Offline: what still works; Send disabled. [Offline]
47. An old library: Later or Update the library. [An old library version]

## What 2B settled beyond 2A's B

For Barry to confirm or change at sign-off:

- **The Steps tab is the model's tree** (asked, 2026-10-27): Built apart (sub-assemblies with "× 5" and their
  sub-steps), then the steps. Clicking a sub-assembly highlights every copy.
- **No command palette** (asked, 2026-10-27).
- **15 colors a theme**, merged from B's palette by role: one accent surface (the user's bubble and selections share
  it), no warn surface (banners sit on the card surface with the icon in the warn color), and the hairline, scrim, and
  shadows mixed from the 15.
- **Enter sends**, Shift+Enter adds a line, and ⌘↩ still sends (section 14's draft had only ⌘↩). Esc closes what's
  open before it stops the agent.
- **Windows**: the title row runs across the window with the menu bar (File, Edit, View, Agent, Help) and the caption
  buttons; Segoe UI Variable; the focus ring in the text color.
- **New screens 2A didn't draw:** a failed download in first run, the project menu, the history open, a tooltip, the
  export toast, Claude's plan limit (4B), and Settings (4A).
- **OpenAI's requirements for Codex on a ChatGPT plan (1E), which B didn't show:** "Using ChatGPT plan" with Manage
  usage by the composer (B's badge said "ChatGPT plan"), the Continue with ChatGPT button in OpenAI's approved format
  (not restyled), OpenAI's usage-limit card, the one-time notice, and the billing row in Settings.
- **Plan usage on the plan badge**, as B drew it: D-36 and section 10 said "status bar meters" and a popover with
  Detailed and Compact views; they now say the badge and its popover (the meters, each reset, Codex's billing state,
  and "Updated 8m ago"). What's fetched, and when, is unchanged.
- **Words:** no exit codes or finding ids on screen, percents without a space ("20%"), and the content guide's
  patterns throughout.
- **The final app icon**: the ant, larger on its plate, with heavier legs for small sizes.
- Not taken up: A's Chat focus layout (not part of the pick). Kept from B as it was: no status bar (the chat's
  status, the plan badge, and About say each thing once).

## Changes Barry asked for at sign-off

None: signed as is.

## Signature

Signed by: Barry Mortenson, as is, through AskUserQuestion in session 2B. Date: 2026-10-27

## After sign-off: design change notes

A change to anything above, once signed, comes as a short note in `docs/design/changes/NNN-<slug>.md` in the same PR
as the change: what changes and why, the screens and components it touches, before and after (screenshots or the
frozen screen's link), and Barry's yes with its date. The frozen files are updated in that PR, and the note is listed
here.

(No changes yet.)
