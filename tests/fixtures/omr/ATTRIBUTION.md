# OMR fixtures

Small official models from the LDraw Official Model Repository (OMR, https://library.ldraw.org/omr), committed
unmodified as validator fixtures (plan section 11, ADR 0003). OMR models are buildable by definition, so validate
must report no findings on any of them (`tests/it/tests/validator_corpus.rs`).

The first five (1D) are the smallest OMR models that validate v0 could judge: built with studs only and using no moved
or obsolete part ids. The five 3A added bring what the connection model must hold: steps and a submodel copy, old
windows with panes and shutters, a door, SNOT, a clip, a bar, a sticker, and a minifigure. Two of them use moved or
obsolete part ids, which validate rightly reports as warnings (snapshotted in `validator_corpus.rs`). The nightly job
validates a larger corpus that isn't committed (`nightly.json`).

Each file keeps its header, author, and license line. The parts they use are in `tests/fixtures/ldraw-mini/`.

| File | Model | Author | License | Source | sha256 |
|---|---|---|---|---|---|
| `544-1_Car.mpd` | 544-1 Basic Set (1981), Car | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1449 | `eb2d67e31507eed4541ba3c2cf5c7ff4124a8b8efdddb06aff1a10a2ebd33c15` |
| `40013-1.mpd` | 40013-1 Halloween Ghost (2010) | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1441 | `5eb3628446875c8654d0636c110353a5982a58889683be2c3a828a1951d5cf17` |
| `544-1_Cow.mpd` | 544-1 Basic Set (1981), Cow | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1449 | `b1f6e303fdb67e1c355fed9c969938b5197205a43f909aca01532cc3df26dc42` |
| `544-1_Airplane.mpd` | 544-1 Basic Set (1981), Airplane (main model) | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1449 | `9f30141e9406039f2238fed8cd70ce1a8a5e5a44acb7f376a173f73641ed9e2b` |
| `566-1_Seaplane.mpd` | 566-1 Basic Set (1981), Seaplane | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1452 | `766379b96c4acc7a7a9cfcdeb1d6e97cda5f2d4204e0302dfe8077ebccb50060` |
| `544-1_House.mpd` | 544-1 Basic Set (1981), House | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1449 | `661e289646d1cc390eeb73b7a2dfc2f2037eeadc52607a17badb175932597ced` |
| `544-1_Ship.mpd` | 544-1 Basic Set (1981), Ship | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1449 | `5ee737aca958f46faafe9c317f8a287ec04e372b343d8b7d9bad6fd5d8bea0ac` |
| `555-2_Farm-Tractor.mpd` | 555-2 Basic Set (1981), Farm Tractor | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1451 | `751eb08d3a4dd27f57d1e164bc14d03b5b9fdb66f9e24d23f66c041abebbee34` |
| `30542-1_Turkey.mpd` | 30542-1 Cute Pug (2018), Turkey | Lasse Deleuran [Lasse Deleuran] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/554 | `3ed23cd67b2296ad64e949f319afb38172a6fe7928e7b64003b857f79cff3c2d` |
| `40011-1.mpd` | 40011-1 Thanksgiving Turkey (2010) | Takeshi Takahashi [RainbowDolphin] | CC BY 2.0 ("Redistributable under CCAL version 2.0") | https://library.ldraw.org/omr/sets/1439 | `885a4b1bf6680ed815c88351bbc441cbf17f0abb6805b0a5a0acfb34af5a88f3` |

The CC BY 2.0 license: https://creativecommons.org/licenses/by/2.0/. LDraw's `CAreadme.txt` is in
`tests/fixtures/ldraw-mini/`.
