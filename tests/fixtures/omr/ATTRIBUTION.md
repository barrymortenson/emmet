# OMR fixtures

Small official models from the LDraw Official Model Repository (OMR, https://library.ldraw.org/omr), committed
unmodified as validator fixtures (plan section 11, ADR 0003). OMR models are buildable by definition, so validate
must report no findings on any of them (`tests/it/tests/validator_corpus.rs`).

These five are the smallest OMR models that validate v0 can judge: built with studs only (no clips, pins, hinges, or
minifigures, which need 3A's connection model) and using no moved or obsolete part ids (which validate rightly
reports as warnings). 3A adds five larger ones, and the nightly job a larger corpus.

Each file keeps its header, author, and license line. The parts they use are in `tests/fixtures/ldraw-mini/`.

| File | Model | Author | License | Source | sha256 |
|---|---|---|---|---|---|
| `544-1_Car.mpd` | 544-1 Basic Set (1981), Car | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1449 | `eb2d67e31507eed4541ba3c2cf5c7ff4124a8b8efdddb06aff1a10a2ebd33c15` |
| `40013-1.mpd` | 40013-1 Halloween Ghost (2010) | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1441 | `5eb3628446875c8654d0636c110353a5982a58889683be2c3a828a1951d5cf17` |
| `544-1_Cow.mpd` | 544-1 Basic Set (1981), Cow | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1449 | `b1f6e303fdb67e1c355fed9c969938b5197205a43f909aca01532cc3df26dc42` |
| `544-1_Airplane.mpd` | 544-1 Basic Set (1981), Airplane (main model) | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1449 | `9f30141e9406039f2238fed8cd70ce1a8a5e5a44acb7f376a173f73641ed9e2b` |
| `566-1_Seaplane.mpd` | 566-1 Basic Set (1981), Seaplane | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1452 | `766379b96c4acc7a7a9cfcdeb1d6e97cda5f2d4204e0302dfe8077ebccb50060` |

The CC BY 2.0 license: https://creativecommons.org/licenses/by/2.0/. LDraw's `CAreadme.txt` is in
`tests/fixtures/ldraw-mini/`.
