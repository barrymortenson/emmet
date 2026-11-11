# ADR 0015: Eval v1 and its thresholds

Status: proposed, 2026-11-11 (session 5A). The thresholds below are section 16.1's, unchanged; "Tuned from the real
runs" is filled in after Barry's G2 hand sessions (H-09), and any change to a threshold is Barry's.

## Context

Plan section 16.1 scores each rung of the ladder once per agent, and says the thresholds are tuned once, in 5A, from
real data, with an ADR to record them. Before 5A, `emmet eval` ran rung 1 and the technique prompts headless on
Barry's API keys, and scored the photo rung from his hand sessions. From G2 every run is a hand session in the app on
his own plans (Barry 2026-11-04), scored afterward with `emmet eval --score`, which sends nothing to an agent.

## Decisions

1. **One scorer for every way a run happens.** A live headless run, a dry run with the fake agents, and a hand
   session's project all end in the same `score`: the model is run and checked at its defaults, then the rung's own
   scores. A hand session's time and turns come from the chat the app keeps in the project (`.emmet/chat/`): the time
   from the first message to the last event, and one turn per tool call.
2. **Rung 2's features, objectively.**
   - *The track loop closes* when some run of track comes back to where it started, joined connector to connector all
     the way round: a ring in the validator's connection graph over the track pieces (53401, 53400, and the 9V switch
     points 75541 and 75542). A loop with a one-stud gap doesn't close.
   - *Platform* and *canopy* are found by name, in a part's id, a sub-assembly's name, or a step's title. The agent
     names its own groups and steps, and the same names are what a person reads in the Steps tab. A geometric test
     (a flat run of tiles beside the track, a roof above figure height over it) was considered and left out: it would
     be a second validator for one rung, and it would fail honest designs it didn't foresee.
   - *The platform's length is a parameter*: one whose name has "platform" or "length" in it and that changes the
     model when set to another value.
3. **Rung 3 against the set.**
   - *BOM F1* counts pieces by part and color, both named as Rebrickable names them: the model's parts list goes
     through the parts data pack (as `emmet bom --format rebrickable` writes it) and is compared with the set's
     Rebrickable inventory, spares left out. A line of 4 where the set has 6 matches 4. Molds of one part count as
     that part (Rebrickable's mold relationships), since a builder ordering parts today gets today's mold.
   - *Placement match* is the share of the set's parts that have a model part of the same LDraw id within one stud
     (20 LDU), each model part matching once; an alias counts as the part it stands for. The agent isn't told where
     the set's origin is or which way it faces, so the set is turned in quarter turns about the vertical and slid to
     where the most parts line up, and the best fit counts. Color isn't part of it: that's the BOM's score.
   - The ground truth is in the repo, outside any project: `tests/fixtures/ladder/rung3/6743-1.mpd` (the OMR file,
     unmodified, CC BY 2.0, credited beside it) and `tests/fixtures/rebrickable-mini/`. H-11's check that the OMR
     main model and the inventory are the same 165 pieces in 69 lines is what lets one stand for the other.
4. **Availability is scored where it can be.** With a parts data pack installed, more than 2 `color_unavailable`
   warnings fail the run; without one the validator can't say, and the line reads n/a and doesn't count. Rung 1's G1
   runs predate the pack and read n/a.
5. **The budget counts, as at G1.** A run over its minutes or turns fails, and so does one that didn't end on its
   own. Rung 3 has no turn budget (section 16.1 gives it minutes only).
6. **The report.** `docs/evals/<date>-rung<n>.md` has a section per agent with the scores, a render, the usage the CLI
   reported, and the questions asked; a rerun adds a section and keeps the first. Each report ends with a trend
   table: every run of that rung in the dated reports, oldest first, a column per score.

## Thresholds

| | Rung 1 | Rung 2 | Rung 3 | Photo |
|---|---|---|---|---|
| Runs and validates with 0 errors | required | required | required | required |
| Parts | ≥ 40 | ≥ 250 | n/a | reported beside the subject's range |
| Features | door, window, sloped roof | track loop closes, platform, canopy | n/a | n/a |
| Parameters that change the model | ≥ 3 | ≥ 5, the platform's length among them | n/a | n/a |
| Against the set | n/a | n/a | BOM F1 ≥ 0.80, placement match ≥ 60 % | n/a |
| Rubric (Barry) | n/a | n/a | n/a | ≥ 4 of 6 per subject, 3 of 4 subjects |
| `color_unavailable` warnings | ≤ 2 | ≤ 2 | ≤ 2 | n/a |
| Budget | ≤ 10 min, ≤ 40 turns | ≤ 30 min, ≤ 120 turns | ≤ 45 min | none |

## Tuned from the real runs

To come with H-09's runs. What the committed projects already say:

- Rung 2, Claude Code's 4B hand session (`tests/fixtures/projects/rung2-claude`): passes every score.
- Rung 2, Codex's 4B hand session (`tests/fixtures/projects/rung2-codex`): the loop closes, the platform and canopy
  are there, and five parameters change the model, but it has 216 parts where the rung asks for 250 (its build
  settings went to Base None partway, so the baseplates came out; `tests/it/tests/ladder.rs`). It fails on size
  alone.

## Consequences

- `emmet eval --rung 2|3` exists, so D-49's stand-ins for "rung 2 scores don't regress" can go: the two rung 2 hand
  sessions are scored in CI (`tests/it/tests/eval.rs`).
- Rung 3's scoring is exact where it's tested: a partial rebuild of the set, turned and moved, scores what its
  missing pieces cost and nothing else.
- 6C's variety eval builds on the same report and trend.
