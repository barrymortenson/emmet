# Ladder ground truth

What rung 3 of the model ladder (`docs/spec/ladder.md`) is scored against. `emmet eval --rung 3` reads the OMR model
here for the placements, and the set's inventory in `tests/fixtures/rebrickable-mini/` for the parts list. The agent
never gets either: an eval's project is made outside the repo, and the prompt gives the set's name, its piece count,
and a description.

The file is from the LDraw Official Model Repository (OMR, https://library.ldraw.org/omr), committed unmodified, with
its header, author, and license line (ADR 0003). `tests/it/tests/eval.rs` checks its sha256 against the nightly
corpus's list.

| File | Model | Author | License | Source | sha256 |
|---|---|---|---|---|---|
| `rung3/6743-1.mpd` | 6743-1 Street Speeder (2009), main model | Marc Giraudet [Mad_Marc] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/media/omr_models/1377/6743-1.mpd | `6d70b766d666788ed4e5082ea1612e484ef613fa2e3c88123069ade350adce93` |

The CC BY 2.0 license: https://creativecommons.org/licenses/by/2.0/. LDraw's `CAreadme.txt` is in
`tests/fixtures/ldraw-mini/`.
