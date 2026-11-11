# The validator, v1

What `emmet validate` (and MCP's `validate`) checks, and how (plan section 11, ADR 0013). The code is
`crates/emmet-core/src/check/`; its tests are `tests/it/tests/validator_corpus.rs`, `validator_mutations.rs`,
`validator_bench.rs`, and `check.rs`, with the goldens in `tests/fixtures/projects/broken-*` and
`tests/fixtures/validator/`.

## Inputs

`build/placements.json`, the library, the parts index, and the parts data pack when one is installed (plan section 12;
`crates/emmet-core/src/parts_data.rs`). Besides `{ schema, model, parts }`
(`docs/spec/dsl.md`), the validator reads three optional keys, which the DSL writes from 3B on:

- `allow`: `[{ "rule", "parts": [ids], "reason" }]`, from `m.allow(rule, ids, reason)`. A finding of that rule whose
  parts are all listed goes under `allowed`, with the reason, instead of `findings`. emmet.toml's `[[check.allow]]`
  does the same (`docs/spec/project-format.md`).
- `ground`: `[ids]`, from `m.ground(id)`: parts the model stands on, besides its lowest level and any baseplate.
- `copies` on a part row: `[{ "id", "step" }]`, one per entry of the part's `submodel` path: the id of that submodel
  copy and the parent's step that places it. Without it, each submodel name counts as one copy.
  `emmet_core::ldraw::placements` fills it for `.mpd` files.

emmet.toml's `[build] age` sets step_size's target: 4 parts a step for 4+, 12 for 18+, and 8 otherwise.

## Connectors

A part's connectors are where it joins others, in its own LDraw frame, in LDCad's snap model: cylinders (studs,
anti-studs, holes, pins, axles, bars) made of sections, each a radius and a length and round, axle-shaped, or a
flexible fit; clips; hinge fingers; and generic plugs with a bounding shape. A shape runs from its position along its
own −Y. Three sources, in order (`connectors.rs`):

1. The primitive table: top-stud and underside-tube primitives the shadow data lacks (logo and open studs, and the
   `stud4` family of tubes: a female cylinder at each tube and its ±10 LDU grid, and the tube's own hole for a bar).
2. The LDCad Shadow Library, `data/connections/ldcad-shadow.ldr` (CC BY-SA 4.0), compiled in.
3. The overrides, `data/connections/*.toml`, each with a test model (`data/connections/README.md`).

A file's connectors are its subfiles', moved into its frame, then its own snap lines. A file the shadow data doesn't
cover falls back to the primitive table; an override with `replace` takes the place of everything else. Connectors
are computed per run, once per part file, and kept for the run.

A part's Technic points (`technic.rs`, 4G) come from its connectors: each cylinder that is a pin hole (a female round
bore a stud's width, open both ends or with a pin hole's rim, at least 8 LDU deep), an axle hole, a pin (a male round
cylinder longer than a stud), an axle, or an axle pin, without a group. A hole drawn in pieces along one line is one
hole, and a pin's halves drawn as their own primitives are the one pin. They're named 1 to n in the order of their
middles in the DSL frame at rot=0, and part_info, `emmet part`, and the DSL (through `build/part-geometry.json`) give
them. `data/connections/technic.toml` covers the Technic parts whose holes are drawn with primitives the shadow data
lacks.

## The connection graph

Two connectors mate when their kinds and genders fit and they're on one line: their axes parallel within about 8° and
apart across them by no more than 0.5 LDU for a stud and 1 LDU for a round joint (D-45). Along the line, a stud goes
all the way into its anti-stud (give or take 0.5 LDU), a round joint at least half way, and a bar or pin anywhere
along a hole or clip that slides; a pin or axle that slides still needs some of its shaft in the hole's bore, or its
collar in the hole's counterbore, not only its tip in the mouth (4G). Radii must fit (no male wider than its hole) and
at least one section must grip.
Hinge fingers mate with the same group's fingers on one line; generic plugs with the same group, opposite gender,
within 1 LDU.

Some pairs hold without connectors (`holds.rs`): a window or door frame and the glass, pane, shutter, or door in it; a
tire and its wheel; a flexible part (hose, cable, chain, string) and what it touches; a sticker and its surface; a
figure's parts and what they touch (figures are posed by hand); a plate wedged on its edge between studs. A part with no
female cylinder (no anti-stud or hole) takes any stud that enters its underside.

The ground is the parts on the model's lowest level (within 1 LDU), any baseplate, and the `ground` parts. In a model
with neither a baseplate nor `ground` parts, a group with nothing of the model under it, within 16 LDU of the lowest
point, stands on the table. In any model, a group whose own tire or wheel rests on top of another part stands (a car
parked on a platform).

## Rules

Findings are sorted by severity (figure_clearance first among the warnings, D-53), then source line, and numbered
`F-0001` on. Every message names the parts, and every
hint says what to change.

| Rule | Severity | Finds | Hint |
|---|---|---|---|
| `unknown_part` | error | a part id the library lacks | search for the part |
| `collision` | error | two parts' collision proxies overlapping by more than 1 LDU (D-46), outside a connected pair's seat, outside the shaft of a pin or axle seated in a hole of the other (an axle through a hub's bore, 4G), and outside a pair one holds the other | how far to move which part (up or down for a vertical overlap); for a pin or axle beside a hole of the other, the hole it missed and how far off it is |
| `floating` | error | a connected group that reaches no ground | the part under it and the gap, in plates; for a pin or axle that misses a hole, the hole it missed (to one side, short of it, or only touching it) and `m.insert` |
| `disconnected_submodel` | error | the outermost submodel copy none of whose parts is joined to or held by a part outside it, inside a parent with parts of its own | where to move the copy |
| `step_order` | error | per scope (the main model, and each copy with its own steps): an element joined to nothing from earlier steps, not resting on the ground, and not the start of a new assembly (or, when all that holds, one resting on parts from later steps) | the step to move it to, or what to move earlier |
| `step_order` | warning | an element whose way in (from above, below, or the side) earlier parts have closed, or that goes in between parts on both sides; a pin or axle in a hole that runs through its mate can come in through it (4G); a part threaded onto an earlier pin's or axle's shaft (its hole around it: a wheel, a gear, a disc, a bush) after earlier parts are on that shaft on both sides of it, half a stud or more away, so it can't slide on from either end (Barry 2026-11-10) | the step to move it to: with the shaft, before both its ends close |
| `step_size` | warning | a step adding more elements than the target (a copy counts once, a figure once per torso) | split the step |
| `weak_connection` | warning | a part held by one or two studs and nothing else that reaches more than 4 studs past them; a part on a pin or an axle (a wheel on its axle) isn't held by a stud (4G) | a support, or more studs |
| `off_grid` | warning | a part joined to nothing whose connector is within 10 LDU of a mate, off sideways, or up or down by other than whole half plates | the move in DSL units |
| `part_unavailable` | warning | a part Rebrickable knows but has no element or set inventory with, in any color: it was never made | a part that was made (its alternates, when Rebrickable lists any) |
| `color_unavailable` | warning | a part in a color it was never made in: no element and no set inventory with it in that color | the closest colors (by RGB) that part was made in |
| `obsolete_part` | warning | a `~Moved to` or obsolete part | the current id |
| `alias_part` | info | an alias | the canonical id |
| `figure_clearance` | warning | a part (not a figure, its accessories, an animal, a door leaf, or a sticker) whose collision proxy is more than 1 LDU inside a space the model keeps clear for a figure (`m.keep_clear`), at figure scale, display, or Any | the smallest move that clears it in a builder's words: raise it (when it hangs into the space from above) or move it sideways, without running into another part; or allow it with a reason |

part_unavailable and color_unavailable give one finding per part × color, naming every placement of it. They need the
parts data pack: without one, or for a part or color with no Rebrickable mapping (ADR 0004's four rules), availability
is unknown and there's no finding. Unknown is never unavailable. The golden is `broken-color-unavailable`, checked with
the pack built from `tests/fixtures/rebrickable-mini/` (`check.rs`).

The first step of a scope is free (its parts are laid out together). An element laid out early and joined later by
parts on top of it is fine. A part moved elsewhere in the library is judged as its target.

## Figure clearance

`figure_clearance` (3E, D-53, `docs/spec/building-rules.md`) checks the spaces a figure needs. A model declares them
with `m.keep_clear(box, why)`: the techniques that place figure doors, seats, porches, and colonnades declare theirs
(`door_frame`'s way through its door, 3 studs out on both sides from a plate above the sill to the door's top;
`bench`'s seat: a stud at each end, 2 in front, and 3 bricks over it), and the agent declares the ones it builds by
hand. They go to `build/placements.json` as `keep_clear: [{ boxes: [[min x, min y, min z, max x, max y, max z]], why,
src }]` in LDraw's frame, a submodel's once per copy. One finding per space names every part inside it. Door frames
found in the parts index make no space: on the 67 official models with 1 x 4 x 6 door frames, a space found that way
gave 269 findings, and still 90 cut to a 1-stud doorway, for counters, stairs, and walls official models put there
(Barry, 2026-10-31). The rule is off when the Size build setting is micro, micropolis, midi, or 1:8. The golden is
`broken-figure-clearance` (a canopy a brick low beside a door, a bench against a column, a wall across a doorway, and
a lamp in a path the model keeps clear); the recall test moves a bench against a post, lowers what stands over a
door's approach a brick, and builds a wall across a doorway (`validator_mutations.rs`).

## Collision proxies

A part's proxy is its body without top studs and underside tubes, as 1 LDU voxels merged into boxes in its own frame
(`proxy.rs`, ADR 0013): vertical rays through the middle of each column, crossings paired, a voxel solid when its
middle is inside. A part with more than 2,048 boxes is filled again at a coarser step. Proxies are built on first use
and cached in the parts index (table `proxies`, index schema 3), so a part is built once per library.

Candidate pairs come from an AABB tree over the parts' world boxes (parry3d). Each pair's boxes are compared in the
first part's frame: exactly when the other is turned by quarter turns, by parry3d's contact query otherwise.

## The summary

The MCP `validate` text, like a compiler's error list:

```text
1 error: 1 floating.
F-0001 error floating model.py:11 at (0, 0, 5): 2 parts at z=5 aren't connected to the rest of the model: no stud, clip, pin, or hinge holds them. They start 1 plate above tower/1 (Brick 2 x 4), which ends at z=4: lower them by 1 plate, or put a part under them.
The full list is in build/validation.json. Fix the errors first, top down (one fix often clears the findings after it), then validate again.
```

The first line counts findings by severity and rule (and allowed ones); then come the first 10 findings, each with its
source line, its position in DSL units, the message, and the hint. The rest are in `build/validation.json`.

## Proof

| Gate | Test | Bar |
|---|---|---|
| Precision | `validator_corpus.rs`: the ten committed OMR fixtures; `corpus_nightly` (ignored; `.github/workflows/nightly.yml`) the models in `tests/fixtures/omr/nightly.json` against the full library | no error findings |
| Recall | `validator_mutations.rs`: seeded faults over the fixtures and the cottage (lift, remove a support, sink, slide, turn, move a step earlier, lump steps, offset a copy, nudge off the grid, unknown part), each with the rule it must trip; the nightly runs it over its corpus | ≥ 95 % overall, ≥ 90 % per error rule |
| Speed | `just bench validator`: warm, release, on the fixtures tiled to 500 and 2,000 parts | ≤ 300 ms and ≤ 1.5 s |
| Goldens | `check.rs` (`broken-*` projects) and `validator_corpus.rs` (`tests/fixtures/validator/`) | exactly the snapshot |
| Overrides | `each_override_holds_in_its_test_model`, `track_joins_exactly` | each test model has no errors, every use of the overridden part connected |
