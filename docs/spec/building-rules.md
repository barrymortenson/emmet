# Building rules

The rules an experienced builder applies without being told (session 3E, D-52). A model that breaks one can still
validate clean as parts: it just isn't built for the figure that lives in it. Each rule says where Emmet enforces it:

- **validator**: the `figure_clearance` rule (D-53), a warning, from the keep-clear boxes techniques and the agent
  declare with `m.keep_clear` (`docs/spec/validator.md`).
- **technique**: a technique's defaults, its arguments, what it returns, and the keep-clear boxes it declares with
  `m.keep_clear` (`docs/techniques/audit.md`).
- **doctrine**: a short rule in the doctrine or a guide the agent reads (`EMMET.md`, `figure-scale.md`, `large.md`,
  `vehicles.md`).
- **render review**: what step 5 of the doctrine's loop tells the agent to look for in the render.

Sources: the guides in `crates/emmet-cli/doctrine/guides/`; methods noted from Barry's instruction books, cited by set
number only (D-48: 43217, 43268, 11387, 43302, 21335); and general building knowledge. Sizes are in the DSL's units:
studs across, plates (and bricks of 3 plates) up.

## The figure

A figure is 4 bricks tall with a hat, 2 studs wide at the shoulders, 1 stud deep, and stands on a 1 x 2 or 2 x 2 spot.
Seated, it reaches about 3 bricks above its seat. Every rule below follows from these sizes.

## 1. Headroom

Anything a figure walks under starts at or above the top of the door beside it: a canopy, an awning, a porch roof, a
shelter's roof, an arcade, a balcony, an archway. For a 1 x 4 x 6 door that's 6 bricks (18 plates) above the floor the
figure stands on; with no door beside it, at least 5 bricks (a figure with a hat, and a brick to spare).

- Enforced by: **technique** (`canopy`, `porch_posts`, and `double_arch_colonnade` take a `height`, 6 bricks by
  default, and refuse under 5; `turned_post_porch` takes `tiers`, 5 by default (20 plates); `lean_to_awning` refuses
  a front edge under 5 bricks over its ground, or under the door top passed as `clears`; `wing_plate_porch_roof`
  refuses under 5 bricks; `door_hood`, `scalloped_awning`, `corbelled_balcony`, and `cross_gable` say where their z
  goes against a door's top and return their `.underside`), **validator** (a door's keep-clear box reaches the door's
  top, so a roof lower than the door over the space in front of it is a finding), **render review** (a walk-under with
  no door beside it).

## 2. A doorway's approach

In front of a door and behind it, a space the door's width, 3 studs out, and from 1 plate above the door's bottom up
to its top stays clear: no wall, post, bench, lamp, sign, or roof edge inside it. A tile or a mat may lie on the floor
there (it's under the space), and a step down from a raised door is fine.

- Enforced by: **technique** (`door_frame` and the other techniques that place a figure door declare this space on
  both sides with `m.keep_clear`, and return the door's top, so the next technique can stand clear of it),
  **validator** (anything inside the space is a finding), **doctrine** (a door frame placed by hand gets the same space,
  declared with `m.keep_clear`). Door frames found in the parts index make no space: official models put a counter, a
  stair, or a wall within a stud of their doors (Barry, 2026-10-31, `docs/decisions.md`).

## 3. Seats

A seat a figure sits on (a bench, a chair, a sofa, a stool) has a stud free at each end, 2 studs clear in front for
the legs, and headroom above it: 3 bricks over the seat with nothing in the way.

- Enforced by: **technique** (`bench`, the seats and chairs, and the vehicle seats declare these spaces with
  `m.keep_clear`), **validator** (anything inside them is a finding), **doctrine** (a seat built by hand gets the same
  spaces, declared with `m.keep_clear`).

## 4. Walkways

Posts, columns, and pillars a figure walks between stand with 3 studs or more clear between them (a figure is 2
wide). No post or column stands in line with a door: in its approach (rule 2).

- Enforced by: **technique** (`canopy` refuses a spacing under 4 or a length under 6 and leaves no last bay under 3
  studs clear; `porch_posts` and `lean_to_awning` refuse a length under 5; `footbridge` stands its posts outside the
  deck; each returns where its columns or posts stand; the porches keep their way in clear when given a `gap`, and
  `double_arch_colonnade` each bay's way through), **validator** (a column in a door's approach or a porch's way in),
  **render review**.

## 5. How parts meet

- A lower roof meets a taller wall under that wall's eaves or on a string course (a ledge a plate proud), never across
  a window or a door. A canopy or an awning beside a door clears the door's top (rule 1).
- Door tops, window tops, and eaves line up along a facade: one line a story.
- Every roof edge meets what's under it: eaves overhang the wall by a stud (or 1 to 2 plates on a small roof), with no
  gap between the roof and the wall top, and no roof edge buried in a wall.
- A seat or a bench goes between columns, centered in the space, not against one (rules 3 and 4).
- Enforced by: **technique** (`door_frame` and `window_frame` at the same `rows` give one line; the roofs overhang by
  default; `canopy` returns its columns so benches can be centered between them), **validator** (a roof across a
  door's opening is inside its approach), **doctrine** and **render review** (the rest).

## 6. Where the rules hold

At figure scale and large, and when the Scale build setting is Any (Emmet builds at figure scale then). Not at micro,
micropolis, midi, or car 1:8: no figure fits there, so `figure_clearance` is off and the guides for those scales don't
teach these rules. At large, a display model with no figure doors or seats has nothing to check; where it has them,
they're for a figure.

- Enforced by: **validator** (it reads the project's Scale build setting), **doctrine** (the guides).

## How a finding reads

`figure_clearance` names the parts, what the space is for, and where it was declared, and its hint is the smallest move
that clears it in a builder's words (preferring one that doesn't run into another part), as in the golden
`broken-figure-clearance`: "The canopy (canopy-beams/2 and 9 more) stands inside the space kept clear for a figure
walking through the door of Shop (declared at model.py:13)." and "Raise the canopy (canopy-beams/2 and 9 more) a brick
so it clears the space for a figure walking through the door of Shop, or allow it with m.allow and a reason the user
would accept." The agent fixes it, or allows it with `m.allow` and a reason the user would accept ("the bench is a
display piece; no figure sits there").