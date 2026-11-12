# G2 runbook: the hand sessions (H-09)

Fourteen hand sessions in the app, once each: rungs 1, 2, and 3 and the four photo subjects, on Claude Code on your
Claude plan and then on Codex on your ChatGPT plan. Each is scored afterward with `emmet eval --score`, which reads
the finished project and its chat and sends nothing to an agent. A failed run gets at most one rerun, after a fix
(D-38).

Order (plan section 22): 5B merges first, this branch is rebased onto it, and the runs happen on this branch so its
PR carries the report.

## What to expect

- **Time.** Rung 1 has 10 minutes, rung 2 has 30, rung 3 has 45; a photo subject has no budget (the e-bike took about
  40 minutes in 4D). At the budgets that's up to about 4 hours an agent, 8 in all, and less if the agents are quick.
- **Cost.** Nothing is billed to an API key: every run is on your plan. Expect a real share of a day's plan usage on
  each; the app's plan badge shows it as you go. Scoring costs nothing.
- **Count.** 14 runs. Don't run one twice unless it failed and something was fixed.

## Each session

1. Open the app: `just dev`
2. New model (⌘N). Name it as the table says. Leave the build settings at Any.
3. Pick the agent in the chat's picker, then paste the prompt from `docs/spec/ladder.md` exactly (the photo rung:
   drop the subject's angle a and type `Build this.`).
4. Answer the agent's questions from the rung's answer script in `docs/spec/ladder.md`. On the photo rung, when it
   asks for another angle, send angle b.
5. Let it finish. Don't steer it beyond the script: a correction you type is part of the run, and its time and turns count.
6. Run the scoring command. It prints the scores and writes the report into `docs/evals/`.

## The runs

Projects land in `~/Documents/Emmet/`. Replace `claude` with `codex` for the second pass, and use the names with
`codex` in them.

| # | Rung | Project name | Scoring command |
|---|---|---|---|
| 1 | 1 | `g2-rung1-claude` | `just eval --rung 1 --agent claude --score ~/Documents/Emmet/g2-rung1-claude` |
| 2 | 2 | `g2-rung2-claude` | `just eval --rung 2 --agent claude --score ~/Documents/Emmet/g2-rung2-claude` |
| 3 | 3 | `g2-rung3-claude` | `just eval --rung 3 --agent claude --score ~/Documents/Emmet/g2-rung3-claude` |
| 4 | Photo: lamp | `g2-lamp-claude` | `just eval --rung photo --subject lamp --agent claude --score ~/Documents/Emmet/g2-lamp-claude` |
| 5 | Photo: e-bike | `g2-e-bike-claude` | `just eval --rung photo --subject e-bike --agent claude --score ~/Documents/Emmet/g2-e-bike-claude` |
| 6 | Photo: Bradbury | `g2-bradbury-claude` | `just eval --rung photo --subject bradbury --agent claude --score ~/Documents/Emmet/g2-bradbury-claude` |
| 7 | Photo: truck | `g2-truck-claude` | `just eval --rung photo --subject truck --agent claude --score ~/Documents/Emmet/g2-truck-claude` |
| 8 to 14 | The same seven | `g2-…-codex` | The same commands with `--agent codex` and the `-codex` project |

The photo commands ask for your three rubric scores (proportions, recognizability, buildability, 0 to 2 each) with
the photo beside the render; `--rubric 2,1,2` gives them on the command line.

## What passes

| Rung | Passes when |
|---|---|
| 1 | 0 validator errors; 40 parts or more with a door, a window, and a sloped roof; 3 or more parameters that change the model; within 10 minutes and 40 turns |
| 2 | 0 validator errors; 250 parts or more with a track loop that closes, a platform, and a canopy; 5 or more parameters that change the model, the platform's length among them; within 30 minutes and 120 turns |
| 3 | 0 validator errors; BOM F1 of 0.80 or more against the set's inventory; 60 % or more of the set's parts in place; within 45 minutes |
| Photo | Per subject: 0 validator errors, built from the photo (the photo is in the first message, before any step), and 4 of 6 or more on your rubric. The rung passes at 3 of the 4 subjects |

With the parts data pack installed, rungs 1 to 3 also need 2 or fewer `color_unavailable` warnings.

## After the runs

Tell the agent they're done. It reads the reports, scrubs each session's transcript into `tests/replay/` with
`scripts/scrub-transcript` (you see each one before it's committed), writes `docs/evals/<date>-g2.md`, and fills in
ADR 0015. If a rung failed, it diagnoses it from the transcript and proposes the smallest fix before the one rerun.
Then: use the app for an hour on a model of your own, run `scripts/terms-check` and `scripts/name-check`, check
section 02's exit criteria, and say "G2 go" or what's missing.
