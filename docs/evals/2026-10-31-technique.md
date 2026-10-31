# Live eval: the technique prompts (2026-10-31)

Session 3E's five technique prompts (`docs/spec/ladder.md`, "Technique prompts", D-52), each run live through
`emmet eval --rung T1` to `T5` on both agents, on the eval API keys, headless, at figure scale on a 32 x 32 base: once on
`main` before 3E's changes (the baseline: `main` and the eval change only) and once at the end of the session (the
branch). 20 runs in all, every one within its budget (20 minutes, 80 turns) and with 0 validator errors. The prompts
tempt the 4B station's mistakes (a roof lower than the door beside it, a bench against a post, a column in a doorway)
and name none of them.

## Summary

The baseline's figure_clearance count is its project rebuilt and validated on the 3E library: `main` had no such rule.
It undercounts what `main` built, because the fixed techniques correct their own defaults on the rebuild (the
stations' canopies stand to the door's top now); the renders show the baseline as `main` built it. The end runs'
projects declare 1 to 11 spaces each (through `door_frame`, `bench`, `scalloped_awning`, `table_and_chairs`, and, in
two runs, the agent's own `m.keep_clear`), so their 0 is a checked 0, not an empty one.

| Prompt | Agent | Baseline (main) | figure_clearance (baseline, on the 3E rule) | End (3E) | figure_clearance (end) | Before | After |
|---|---|---|---|---|---|---|---|
| T1: The station | Claude Code | 133 parts, 0 errors | 3 | 163 parts, 0 errors | **0** (3 spaces declared) | ![before](2026-10-31-t1-claude-before.png) | ![after](2026-10-31-t1-claude-after.png) |
| T1: The station | Codex | 154 parts, 0 errors | 0 | 196 parts, 0 errors | **0** (3 spaces declared) | ![before](2026-10-31-t1-codex-before.png) | ![after](2026-10-31-t1-codex-after.png) |
| T2: A corner shop with an awning | Claude Code | 278 parts, 0 errors | 0 | 330 parts, 0 errors | **0** (2 spaces declared) | ![before](2026-10-31-t2-claude-before.png) | ![after](2026-10-31-t2-claude-after.png) |
| T2: A corner shop with an awning | Codex | 374 parts, 0 errors | 0 | 377 parts, 0 errors | **0** (1 spaces declared) | ![before](2026-10-31-t2-codex-before.png) | ![after](2026-10-31-t2-codex-after.png) |
| T3: A cottage with a porch | Claude Code | 197 parts, 0 errors | 1 | 218 parts, 0 errors | **0** (2 spaces declared) | ![before](2026-10-31-t3-claude-before.png) | ![after](2026-10-31-t3-claude-after.png) |
| T3: A cottage with a porch | Codex | 312 parts, 0 errors | 0 | 440 parts, 0 errors | **0** (2 spaces declared) | ![before](2026-10-31-t3-codex-before.png) | ![after](2026-10-31-t3-codex-after.png) |
| T4: A bus stop | Claude Code | 146 parts, 0 errors | 0 | 213 parts, 0 errors | **0** (1 spaces declared) | ![before](2026-10-31-t4-claude-before.png) | ![after](2026-10-31-t4-claude-after.png) |
| T4: A bus stop | Codex | 74 parts, 0 errors | 0 | 73 parts, 0 errors | **0** (2 spaces declared) | ![before](2026-10-31-t4-codex-before.png) | ![after](2026-10-31-t4-codex-after.png) |
| T5: A café with seats outside | Claude Code | 363 parts, 0 errors | 0 | 425 parts, 0 errors | **0** (11 spaces declared) | ![before](2026-10-31-t5-claude-before.png) | ![after](2026-10-31-t5-claude-after.png) |
| T5: A café with seats outside | Codex | 193 parts, 0 errors | 0 | 200 parts, 0 errors | **0** (1 spaces declared) | ![before](2026-10-31-t5-codex-before.png) | ![after](2026-10-31-t5-codex-after.png) |

**figure_clearance: 4 in the baseline (all Claude Code: T1's booth counter in its doorway and a column against each
bench; T3's porch post against its bench), 0 at the end, on both agents.** Both baseline stations repeat the 4B
mistakes in their renders: the canopy low against the booth, the benches against the columns. Asked for the same
station at the end, both agents centered the benches between the canopy's columns and stood the canopy at the booth's
door height, without the prompt mentioning either (What good looks like 1).

Questions: none were asked in any run. Usage, where the CLI reports it: the end runs took 0.3 to 2.3 million input
tokens each (Claude Code 1.7 to 2.3, Codex 0.3 to 0.7); each section below gives its own, and the providers' consoles
have the cost.

Barry's judgment of the before and after renders (H-27, 2026-10-31, through AskUserQuestion): better; approved.

## The baseline, on main

### Rung T1: The station, Claude Code — pass

Claude Code 2.1.316, model claude-opus-5-5, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 133 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 2 min 31 s (≤ 20 min), 32 turns (≤ 80), ended: completed |

![The model Claude Code built](2026-10-31-t1-claude-before.png)

Usage: 1105844 input and 13429 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> I built the station on the green 32 × 32 baseplate. The platform is 24 × 8 studs and 4 plates high, with a yellow safety line along the front edge. The 6 × 6 ticket booth stands at the left end. It's 7 bricks tall, with a 1 × 4 × 6 door that opens under the canopy, a ticket window with a counter behind it, a side window and a gable roof. The canopy covers the other 18 studs, with a column every 6 studs and two benches under it facing the front. That's 133 parts, and the checks find no problems. I also checked the shortest (16) and longest (28) platforms and every canopy and color option.
> 
> There are three knobs:
> - **Platform length:** 16 to 28 studs. The booth always takes 6 of them and the canopy gets the rest.
> - **Canopy style:** flat or gable.
> - **Station colors:** heritage (tan and dark green with dark red roofs), seaside (white and dark blue) or red brick (dark red and black). In the seaside scheme the door frame is white, because that part was never made in dark blue.
> 
> The 10-stud strip in front of the platform is empty, ready for track. I can add a straight run of track there and a pair of platform lamps if you'd like.

### Rung T1: The station, Codex — pass

Codex 0.160.1, model gpt-6.1-sol, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 154 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 1 min 1 s (≤ 20 min), 35 turns (≤ 80), ended: completed |

![The model Codex built](2026-10-31-t1-codex-before.png)

Usage: 450455 input and 3503 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> Built on a green 32 × 32 baseplate with two sheltered benches and a ticket booth with a door: 154 parts, 0 errors or warnings.
> 
> Controls include platform length (24–28 studs), canopy style (flat or gable), and station colors (three palettes).

### Rung T2: A corner shop with an awning, Claude Code — pass

Claude Code 2.1.316, model claude-opus-5-5, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 278 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 6 min 36 s (≤ 20 min), 48 turns (≤ 80), ended: completed |

![The model Claude Code built](2026-10-31-t2-claude-before.png)

Usage: 2327623 input and 36885 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> I built the corner shop on the green 32 × 32 baseplate: 16 × 14 studs and 9 bricks tall to the roof edge, 278 parts, and the checks come back clean. Sidewalks run along the front and left edges, both of those walls have shop windows, the door is on the front near the corner, and a striped awning (your color alternating with white) wraps the corner over the windows and the door. The flat roof has a low wall around it with a sign on the front, and I added two street lamps, a bench and a counter inside.
> 
> The panel has three controls: shop_width is a slider from 12 to 24 studs, and awning_color and wall_color are color pickers rather than sliders. I checked both ends of the width range plus a few colors, and all came back clean. If you pick a rare color, the checks will flag any brick that was never made in it.
> 
> Next, I could make the roof lift off so you can see inside the shop.

### Rung T2: A corner shop with an awning, Codex — pass

Codex 0.160.1, model gpt-6.1-sol, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 374 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 1 min 52 s (≤ 20 min), 36 turns (≤ 80), ended: completed |

![The model Codex built](2026-10-31-t2-codex-before.png)

Usage: 526832 input and 7167 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> Built a 20 × 16 corner shop on the green 32 × 32 baseplate, with windows on both adjoining walls, a door, wraparound awning, and flat roof with a SHOP sign.
> 
> 374 parts; validation is clean. Controls: shop width (16–24 studs), awning color, and wall color.

### Rung T3: A cottage with a porch, Claude Code — pass

Claude Code 2.1.316, model claude-opus-5-5, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 197 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 5 min 1 s (≤ 20 min), 40 turns (≤ 80), ended: completed |

![The model Claude Code built](2026-10-31-t3-claude-before.png)

Usage: 1458947 input and 26335 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> I've built the cottage on the green baseplate. By default it's 14 × 10 studs with walls 7 bricks tall. A 4-stud-deep porch runs the full width of the front, and the front door opens onto it. The porch has a bench to the left of the door and its own roof standing on four white posts, separate from the house's sloped gable roof, which lifts off. The model is 197 parts and passes every check with no errors or warnings. It also passes at the slider extremes I tried (width 10 with porch 3, width 20 with porch 6) and with other colors.
> 
> The four sliders are cottage width (10–20 studs), porch depth (3–6 studs), wall color (default Tan), and roof color (default Dark Red). Emmet normally keeps sizes and colors off the slider panel, but you asked for them, so I added them and noted that in the model's notes so they stay.

### Rung T3: A cottage with a porch, Codex — pass

Codex 0.160.1, model gpt-6.1-sol, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 312 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 1 min 10 s (≤ 20 min), 34 turns (≤ 80), ended: completed |

![The model Codex built](2026-10-31-t3-codex-before.png)

Usage: 369370 input and 4268 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> Built an 18 × 12 cottage with a covered 18 × 6 porch and bench on the green 32 × 32 baseplate: 312 parts, zero validation findings.
> 
> Controls include cottage width, porch depth, wall color, and roof color; both size extremes also validate cleanly.

### Rung T4: A bus stop, Claude Code — pass

Claude Code 2.1.316, model claude-opus-5-5, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 146 parts, 2 findings (0 errors): step_size 2 |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 4 min 56 s (≤ 20 min), 39 turns (≤ 80), ended: completed |

![The model Claude Code built](2026-10-31-t4-claude-before.png)

Usage: 1535984 input and 25732 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> The bus stop is built on the 32 × 32 baseplate: 146 parts, 0 errors. Along the front is an 8-stud-deep sidewalk. On it stand a glass shelter 4 studs deep and 5 bricks tall inside, with a roof, back wall and a bench, plus a printed round "BUS STOP" sign on its pole by the curb, a trash can, and a street lamp I added. I also put two trees, two bushes and a flower bed on the grass behind.
> 
> The three sliders are `shelter_length` (6 to 16 studs, in steps of 2, default 10), `roof_color`, and `ad_panel`, which puts an ice-cream poster panel in the right end of the back wall, facing the street. Emmet's guidelines normally keep sizes and colors off the sliders, but I added them because you asked. I checked the shortest shelter with the panel and the longest without it in another roof color, and both come out with no errors.
> 
> Validate's only warnings are that a few sidewalk and roof steps have 9 to 15 parts, over its 8-per-step target. They're plain runs of the same tile or plate, so I left them.

### Rung T4: A bus stop, Codex — pass

Codex 0.160.1, model gpt-6.1-sol, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 74 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 0 min 47 s (≤ 20 min), 25 turns (≤ 80), ended: completed |

![The model Codex built](2026-10-31-t4-codex-before.png)

Usage: 280291 input and 2933 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> Built the bus stop on a green 32 × 32 baseplate with all requested features: 74 parts, zero validation errors or warnings.
> 
> Controls include Shelter length (12–20 studs), Roof color, and Advertising panel on/off; both length extremes also validate cleanly.

### Rung T5: A café with seats outside, Claude Code — pass

Claude Code 2.1.316, model claude-opus-5-5, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 363 parts, 10 findings (0 errors): step_size 10 |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 6 min 52 s (≤ 20 min), 58 turns (≤ 80), ended: completed |

![The model Claude Code built](2026-10-31-t5-claude-before.png)

Usage: 3497748 input and 32507 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> I built the café: a 24 × 14 stud building about 9 bricks tall, 363 parts in 58 steps, with 0 errors in validate. It has a green door set off center, two big white shop windows, a red-and-white striped scalloped awning across the whole front, and a flat roof that lifts off to show a counter and coffee machine. Out front are four tables, each with two chairs, on a 16-stud-deep sidewalk.
> 
> The sliders are tables (0–6), awning_color, awning_stripe and wall_color. The color sliders pick from short lists, and I checked every color on them against the parts that use it. I dropped three choices because some parts were never made in them: coral and dark green for the awning, and medium azure for the walls. Validate still gives 10 warnings that some steps have 9–14 parts instead of the target 8. These are rows of identical sidewalk tiles and steps that Emmet's ready-made window and roof pieces set themselves.
> 
> Next, I could add a sign over the awning, or a lamp post and a tree on the bare grass at the sides.

### Rung T5: A café with seats outside, Codex — pass

Codex 0.160.1, model gpt-6.1-sol, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 193 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 1 min 3 s (≤ 20 min), 29 turns (≤ 80), ended: completed |

![The model Codex built](2026-10-31-t5-codex-before.png)

Usage: 402489 input and 3925 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> Built the café on a green 32 × 32 baseplate with big front windows, a door, striped awning, and three outdoor tables with chairs: 193 parts, no validation findings.
> 
> Controls include a 0–4 table slider and color pickers for both awning stripes and the walls.

## The end, on the 3E branch

### Rung T1: The station, Claude Code — pass

**Prompt:** Build a small train station on a green 32 × 32 baseplate: a platform with a canopy over it, two benches under the canopy, and a ticket booth with a door at one end of the platform. Give me sliders for the platform length, the canopy style, and the station colors.

Claude Code 2.1.316, model claude-opus-5-5, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 163 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 4 min 15 s (≤ 20 min), 43 turns (≤ 80), ended: completed |

![The model Claude Code built](2026-10-31-t1-claude-after.png)

Usage: 1715252 input and 20589 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> The station sits on the 32 × 32 baseplate. The platform is 26 × 8 studs with a yellow edge line beside a straight run of track. A canopy on columns covers two benches, and at the right end is a 6 × 6 ticket booth, 7 bricks tall, with a door onto the platform and a ticket window on each side. The model has 163 parts and validation finds no problems.
> 
> Your sliders are platform length (22 to 30 studs), canopy style (flat or gable; the booth roof changes to match), and station colors (classic green, red brick, seaside blue or modern gray). I also added a track on/off switch.
> 
> I tested the extreme settings and every color scheme, and they all pass validation. Seaside blue uses Sand Blue window frames, because the window part was never made in Blue. I only had a picture of the default flat-roof version to check. The gable settings passed validation, but I haven't looked at a picture of them.

### Rung T1: The station, Codex — pass

**Prompt:** Build a small train station on a green 32 × 32 baseplate: a platform with a canopy over it, two benches under the canopy, and a ticket booth with a door at one end of the platform. Give me sliders for the platform length, the canopy style, and the station colors.

Codex 0.160.1, model gpt-6.1-sol, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 196 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 0 min 51 s (≤ 20 min), 25 turns (≤ 80), ended: completed |

![The model Codex built](2026-10-31-t1-codex-after.png)

Usage: 344373 input and 3077 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> Built the station on a green 32 × 32 baseplate with two sheltered benches and a ticket booth with a door: 196 parts, zero validation findings.
> 
> Controls include platform length (24–28 studs), canopy style (flat or gabled), and station colors (three palettes).

### Rung T2: A corner shop with an awning, Claude Code — pass

**Prompt:** Build a corner shop on a green 32 × 32 baseplate: shop windows along two walls that meet at the corner, a door, an awning over the windows and the door, and a flat roof with a sign. Give me sliders for the shop's width, the awning color, and the wall color.

Claude Code 2.1.316, model claude-opus-5-5, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 330 parts, 2 findings (0 errors): step_size 2 |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 7 min 42 s (≤ 20 min), 57 turns (≤ 80), ended: completed |

![The model Claude Code built](2026-10-31-t2-claude-after.png)

Usage: 2270049 input and 41662 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> I've built the corner shop: 16 × 12 studs by default, walls about 8 bricks tall to the lift-off flat roof, 330 parts. Validate shows no errors, only two step-size warnings for the sidewalk tile runs, which I left because each is a single plain run of the same tile. Shop windows line both street walls, with the door beside the corner, a striped awning running along both walls over the windows and the door, and the sign standing on the roof's front edge.
> 
> The sliders are `shop_width` (12 to 24 studs), `awning_color` and `wall_color`; the awning color also colors the sign's frame. The color sliders are lists rather than free pickers: 14 awning colors and 8 wall colors, each one a color every part it goes on was actually made in. Free pickers let you choose colors that some of the long bricks or awning pieces never came in.
> 
> I also added a glass display counter inside, a street lamp on each sidewalk, and a tree and a bench in the yard.

### Rung T2: A corner shop with an awning, Codex — pass

**Prompt:** Build a corner shop on a green 32 × 32 baseplate: shop windows along two walls that meet at the corner, a door, an awning over the windows and the door, and a flat roof with a sign. Give me sliders for the shop's width, the awning color, and the wall color.

Codex 0.160.1, model gpt-6.1-sol, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 377 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 1 min 59 s (≤ 20 min), 48 turns (≤ 80), ended: completed |

![The model Codex built](2026-10-31-t2-codex-after.png)

Usage: 739174 input and 7296 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> Built an 18 × 16-stud corner shop on a green 32 × 32 baseplate, with windows on two adjoining walls, a door, wraparound awning, and flat roof with a SHOP sign. It has 377 parts and validates with 0 errors or warnings.
> 
> Controls are available for shop width (16–24 studs), awning color, and wall color.

### Rung T3: A cottage with a porch, Claude Code — pass

**Prompt:** Build a cottage with a covered front porch on a green 32 × 32 baseplate: the front door opens onto the porch, the porch has its own roof on posts, the house has a sloped roof, and there's a bench on the porch. Give me sliders for the cottage's width, the porch depth, and the wall and roof colors.

Claude Code 2.1.316, model claude-opus-5-5, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 218 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 6 min 45 s (≤ 20 min), 41 turns (≤ 80), ended: completed |

![The model Claude Code built](2026-10-31-t3-claude-after.png)

Usage: 1693031 input and 36621 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> The cottage is built: 14 × 8 studs with a 4-stud porch, 218 parts in 56 steps, and the model check finds no problems. The door opens onto the porch, which has its own sloped roof on white posts and a bench under the front window. The house has a Dark Red gable roof over Tan walls.
> 
> Your four sliders are cottage width, porch depth, wall color and roof color. Width runs 14 to 20 studs in steps of 2, because odd widths need a roof-ridge piece that was never made in most roof colors. Porch depth runs 3 to 6 studs. Both ends of each range pass the check, but I haven't seen the widest setting in a picture, which is the only one with a third post in the middle of the porch.

### Rung T3: A cottage with a porch, Codex — pass

**Prompt:** Build a cottage with a covered front porch on a green 32 × 32 baseplate: the front door opens onto the porch, the porch has its own roof on posts, the house has a sloped roof, and there's a bench on the porch. Give me sliders for the cottage's width, the porch depth, and the wall and roof colors.

Codex 0.160.1, model gpt-6.1-sol, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 440 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 1 min 9 s (≤ 20 min), 35 turns (≤ 80), ended: completed |

![The model Codex built](2026-10-31-t3-codex-after.png)

Usage: 470389 input and 4850 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> Built a 440-part cottage on the green 32 × 32 baseplate, with a sloped lift-off roof, front door onto the covered porch, four posts, and a bench. Width and porch-depth sliders plus wall and roof color controls are available; validation reports no errors or warnings, including at both size extremes.

### Rung T4: A bus stop, Claude Code — pass

**Prompt:** Build a bus stop on a green 32 × 32 baseplate: a stretch of sidewalk, a shelter with a roof and a back wall, a bench in the shelter, a bus stop sign on a pole, and a trash can. Give me sliders for the shelter's length, the roof color, and whether the shelter has an advertising panel.

Claude Code 2.1.316, model claude-opus-5-5, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 213 parts, 7 findings (0 errors): step_size 7 |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 4 min 56 s (≤ 20 min), 41 turns (≤ 80), ended: completed |

![The model Claude Code built](2026-10-31-t4-claude-after.png)

Usage: 1702225 input and 25387 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> I built the bus stop: 213 parts in 56 steps. A 12-stud sidewalk with a curb runs along the front of the baseplate. The shelter sits on it, 5 studs deep and 7 bricks tall, with a glazed back wall, a dark red roof, a bench centered inside, a printed round "BUS STOP" sign near the curb and a green trash can. I added two trees and three bushes on the grass behind. Validate shows no errors, and the shortest shelter without the ad panel and the longest with it both check clean.
> 
> The sign is an old printed part on its round base, and Emmet has no data on how those two connect, so I marked their "floating" and "overlapping" findings as allowed rather than fixed. The only warnings left are that the sidewalk steps have 10 to 11 tiles each, a bit over the target of 8.
> 
> Your three knobs are `shelter_length` (8 to 16 studs, default 12), `roof_color`, and `ad_panel`, which puts a yellow tiled advertising panel on the right end of the shelter facing oncoming traffic; with it off, that end is glass like the left one.

### Rung T4: A bus stop, Codex — pass

**Prompt:** Build a bus stop on a green 32 × 32 baseplate: a stretch of sidewalk, a shelter with a roof and a back wall, a bench in the shelter, a bus stop sign on a pole, and a trash can. Give me sliders for the shelter's length, the roof color, and whether the shelter has an advertising panel.

Codex 0.160.1, model gpt-6.1-sol, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 73 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 1 min 16 s (≤ 20 min), 37 turns (≤ 80), ended: completed |

![The model Codex built](2026-10-31-t4-codex-after.png)

Usage: 499671 input and 4336 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> Built on a green 32 × 32 baseplate with all requested features: 73 parts, zero validation findings, including both shelter length extremes.
> 
> Controls: shelter length (12–20 studs), roof color, and advertising panel on/off.

### Rung T5: A café with seats outside, Claude Code — pass

**Prompt:** Build a small café on a green 32 × 32 baseplate: a one-story building with a door and big front windows, a striped awning over the front, and tables with chairs outside on the sidewalk. Give me sliders for the number of tables, the awning colors, and the wall color.

Claude Code 2.1.316, model claude-opus-5-5, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 425 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 5 min 29 s (≤ 20 min), 45 turns (≤ 80), ended: completed |

![The model Claude Code built](2026-10-31-t5-claude-after.png)

Usage: 2085220 input and 26849 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> I built the café: a 24 × 12 stud building about 9 bricks tall on the 32 × 32 baseplate, 425 parts. It has a center door, two big three-pane shop windows, a full-width red-and-white scalloped awning, and four round tables for two on the sidewalk. Inside are a checkered floor, a glass display counter and a table, and the flat roof lifts off. At the default settings the check finds no errors or warnings, with one large roof step allowed on purpose.
> 
> You get four controls: `tables` is a 0–4 slider, while `awning_color`, `awning_stripe` and `wall_color` are color pickers rather than sliders. Some colors were never made in every part the café uses, and the check flags those. Medium Azure walls and a Dark Green awning both got flagged when I tried them. Tan, Sand Green, red, white, yellow and dark blue all came out clean. A signboard on the strip of wall above the awning would be a good next step.

### Rung T5: A café with seats outside, Codex — pass

**Prompt:** Build a small café on a green 32 × 32 baseplate: a one-story building with a door and big front windows, a striped awning over the front, and tables with chairs outside on the sidewalk. Give me sliders for the number of tables, the awning colors, and the wall color.

Codex 0.160.1, model gpt-6.1-sol, billed to API key. Emmet 0.1.0.

| Score | Required | Result | Detail |
|---|---|---|---|
| Runs and validates with 0 errors | yes | pass | 200 parts, 0 findings (0 errors) |
| figure_clearance | yes | pass | 0 findings |
| Budget | yes | pass | 1 min 17 s (≤ 20 min), 33 turns (≤ 80), ended: completed |

![The model Codex built](2026-10-31-t5-codex-after.png)

Usage: 501547 input and 4712 output tokens (where the CLI reports them; the provider's console has the cost).

Questions: none.

The agent's last message:

> Built your café on a green 32 × 32 baseplate with big front windows, a door, striped awning, and outdoor seating—200 parts, zero validation findings.
> 
> Controls include number of tables (0–3), both awning colors, and wall color.
