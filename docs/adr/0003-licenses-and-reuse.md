# ADR 0003: Licenses and reuse

Status: accepted, 2026-10-24 (session 0B).

## Context

Emmet ships LDraw part files in a test fixture, may ship connector data derived from the LDCad shadow library, uses
Rebrickable's catalog, uses three.js's LDrawLoader, and sits next to several open projects that do similar things.
Plan section 19 (rule 6, rule 10) says: learn from other projects, and copy code only where the license allows and
Barry agrees. Every license below was read in its own file or page on 2026-10-24; GitHub repos are pinned to the commit
read.

## Findings

### LDraw parts library: CC BY 4.0 and CC BY 2.0

- **Source:** `CAreadme.txt` inside the 2026-09 `complete.zip` (sha256 `f3e496d0…a4dd5f2`) and at
  https://library.ldraw.org/library/official/CAreadme.txt. Parts carry one of three `!LICENSE` lines: CC BY 4.0;
  "CC BY 2.0 and CC BY 4.0" (parts edited before 2023-03-05 by some authors who didn't re-affirm the agreement); or
  the deprecated "CCAL version 2.0", which the readme says to read as CC BY 2.0. The full texts ship as
  `CAlicense.txt` (CC BY 2.0) and `CAlicense4.txt` (CC BY 4.0).
- **Attribution:** the readme lists five items (creator, URL of the work, license name, license URL, whether it was
  modified) and says items 1–4 are given by each file's header or the readme itself; marking changes is ours.
- **Not derivative works:** renders made from the library, and model files that only reference official parts. So
  Emmet's `.mpd` output and renders carry no LDraw license obligation.
- **Note:** https://www.ldraw.org/legal-info still describes CCAL 2.0 only (its footer reads 2002–2020). The readme
  in the library is current and is what Emmet follows.
- **Decision:** `tests/fixtures/ldraw-mini/` keeps every file byte for byte with its header, ships the three license
  texts and a generated `ATTRIBUTION.md` (source URL, release, sha256, licenses, "no changes", every file with its
  author and license line), and `scripts/ldraw-mini-check.mjs` enforces that in CI. `NOTICE` keeps its LDraw
  paragraph. The fixture's 505 part and primitive files: 495 CC BY 4.0, 10 "CC BY 2.0 and CC BY 4.0".
- **Part authors' names:** the headers name each part's author, and CC BY requires keeping them. Barry added a second
  exception to the no-real-names invariant for exactly this (2026-10-24): third-party authors' names a license
  requires, in AGENTS.md, plan section 25, and agent rule 7.

### LDraw OMR models: committable as fixtures

- **Source:** the OMR specification (https://www.ldraw.org/article/593.html, updated 22-Nov-2023) gives new models the
  header `0 !LICENSE Licensed under CC BY 4.0 : see CAreadme.txt`; the OMR rules
  (https://www.ldraw.org/docs-main/official-model-repository-omr/rules-and-procedures-for-the-official-model-repository.html)
  say legacy models use the dual 2.0/4.0 line unless the author re-licensed, and embedded parts keep their own line.
  The five shortlist files downloaded in 0B carry "Redistributable under CCAL version 2.0" (CC BY 2.0).
- **Decision (Q-09):** yes. Small OMR models may be committed as fixtures (`tests/fixtures/omr/`, 3A) under the same
  rule as ldraw-mini: unmodified, headers intact, an attribution file naming each model's author and license. Renders
  of OMR models (such as the rung 3 shortlist in `docs/spec/ladder/`) are not derivative works, and the ladder spec
  credits each model's author anyway.

### LDCad shadow library: CC BY-SA 4.0, shipped as its own data file (Q-01)

- **Source:** https://github.com/RolandMelkert/LDCadShadowLibrary at `9b1131f` (2026-09-10): `LICENSE.md` is the
  Creative Commons Attribution-ShareAlike 4.0 legal code, and every shadow file says
  `0 !LICENSE CC BY-SA 4.0, see LICENSE.md`.
- **Format:** LDraw "patch" files named like the parts they extend, holding `!LDCAD` meta lines that LDCad appends to
  the part at load: `SNAP_CYL` (studs, anti-studs, bars, pins, axle holes, as cylinders with gender and caps),
  `SNAP_CLP` (clips), `SNAP_FGR` (finger hinges), `SNAP_GEN` (generic), `SNAP_INCL` (include another shadow),
  `SNAP_CLEAR`, and `MIRROR_INFO`. The syntax is documented at https://www.melkert.net/LDCad/tech/meta.
- **Coverage:** 3,349 part files and 87 primitive files (including `stud.dat`, `stud2.dat`, `stud3.dat`, `stud4.dat`)
  against about 20,300 real parts in the 2026-09 library. Of ldraw-mini's 141 real parts, 94 have a shadow file,
  among them the clip (4081b), hinge plates (2429, 2430), bar (87994), pin (4274), pin hole (6541), jumpers (3794b,
  15573), and brackets (99780, 99781, 3956). Track (53400, 53401) and the 9V switches have none, so they need Emmet's
  own overrides either way. (Counts from the repo tree at `9b1131f`; the command is under "Rerun".)
- **What ShareAlike means here:** reading the shadow files at run time is use, not distribution. But Emmet plans to
  ship a connector table built from them (in the parts index or data pack); that table would be an adaptation, which
  must be shared under CC BY-SA 4.0 (or a later or compatible license) with attribution. It could ship as its own
  data file under CC BY-SA 4.0 next to the AGPL code, which is a separate work aggregated with the app, without
  changing Emmet's license.
- **Decision (Barry 2026-10-24, Q-01):** use it. The connector table built from it ships as its own data file under
  CC BY-SA 4.0, with attribution to the LDCad Shadow Library (author line, URL, license, and "converted to Emmet's
  connector table"), separate from the AGPL code, and `NOTICE` names it when it first ships (3A). Overrides in
  `data/connections/` cover what it misses. Rejected: building the table on the user's machine from a first-run
  download, and not using it.

### Rebrickable catalog: free with credit, no automated site traffic

- **Source:** https://rebrickable.com/downloads/ and https://rebrickable.com/terms/, read in a browser on 2026-10-24
  (both answer scripts with a Cloudflare challenge). The phrases are in `docs/terms/quotes.json`.
- The downloads are "Free for personal and commercial use, provided you acknowledge Rebrickable as your data source",
  and may be fetched automatically "at most once a day". The Terms ban using automation tools to automate traffic to
  the site (5.3) apart from the downloads, ban scraping MOC data, and say Rebrickable content may not be used to
  train AI models.
- **Decision:** D-25 stands (Q-02 stays settled): one CI fetch a week, credit wherever the data appears, no MOC data,
  no images, no training. `scripts/terms-check` lists Rebrickable's pages as a hand checklist and never fetches them.
  `tests/fixtures/rebrickable-mini/` carries the credit, and so does `NOTICE` now. Q-14 (small facts reaching the
  user's AI provider in tool results) is settled by reading: the clause is about training models on Rebrickable
  content, and Emmet trains nothing. Barry dropped the planned note to Rebrickable (H-24 not needed, 2026-10-24).

### three.js and LDrawLoader: MIT

- **Source:** `LICENSE` in the `three` 0.186.1 package ("The MIT License … Copyright © 2010-2026 three.js authors");
  `examples/jsm/loaders/LDrawLoader.js` and `examples/jsm/materials/LDrawConditionalLineMaterial.js` are part of it.
- **Decision:** use it as a dependency (section 06 names it). The license notice goes into the third-party notices
  file that 6A generates. In r186, LDrawLoader needs `setConditionalLineMaterial(LDrawConditionalLineMaterial)`
  before it loads anything (found by the ADR 0005 spike).

### Projects next to Emmet

| Project (commit read) | License | What it is | What Emmet may take |
|---|---|---|---|
| [anteloc/ldraw-nova](https://github.com/anteloc/ldraw-nova) (`00f23f6`) | Code AGPL-3.0 (`LICENSE`); research notes and docs CC BY-SA 4.0 (`ATTRIBUTION.md`, `CC-BY-SA-4.0`), adapted in part from the author's ldraw-mecha | A Python toolkit where an agent writes JSON plans, builds MPDs, checks geometry, renders through LeoCAD, and compares parts lists; ships connector metadata and reference constructions | Ideas. The code's license matches Emmet's, so reuse is legally possible with attribution, and its docs and data would bring ShareAlike; either needs Barry's yes first |
| [jonx/BrickBuilderMCP](https://github.com/jonx/BrickBuilderMCP) (`f397a80`) | MIT | An MCP server with 57 tools that has an LLM design LDraw models with a connection model, validation, and rendering | Ideas (tool grouping, validation feedback). Code under MIT with its notice, if Barry agrees |
| [ElRashoMacuin24/brickbuilder-mcp](https://github.com/ElRashoMacuin24/brickbuilder-mcp) (`3f9c72e`) | MIT | An MCP server for designing from photos and exporting LDraw | Ideas for 4D (photo to model) |
| [datakurre/brick-mcp](https://github.com/datakurre/brick-mcp) (`90c97bb`) | No license file, so all rights reserved | An MCP server that edits Studio `.io` and LDraw files | Ideas only (Studio files, 8G) |
| [musharna/ldraw-mcp](https://github.com/musharna/ldraw-mcp) (`eb087cf`) | MIT | An MCP server that renders LDraw files through headless Blender and ImportLDraw | Ideas (rendering for the agent); Emmet renders through its own viewer (D-24) |
| [BaLaurent/ClawdCAD](https://github.com/BaLaurent/ClawdCAD) (`ebd6363`) | GPL-3.0 | An Electron app with Monaco, a three.js viewer, and Claude for OpenSCAD; its README says it's no longer developed | Ideas (editor, viewer, and agent in one window) |
| [AnyiWang/OpenCovibe](https://github.com/AnyiWang/OpenCovibe) (`b070b1d`) | Apache-2.0 (`LICENSE`, `NOTICE`) | A local Tauri desktop app around Claude Code and Codex (app-server mode with an `exec` fallback) | Ideas: the closest architecture to Emmet's agent bridge. Code under Apache-2.0 keeps its NOTICE, if Barry agrees |

The two "brickbuilder-mcp" repos both match the name in the plan; both are listed. "brick-mcp" in the plan is
datakurre's (the other search hits are for the unrelated Bricks Builder WordPress plugin).

**Decision:** 0B copies no outside code. Every later session follows the default stance: read for ideas, and ask Barry
before copying anything, with the license and the attribution it needs.

## Rerun

```sh
# LDraw: the readme inside the pinned library (this URL serves the newest release; compare the sha256 with the pin)
curl -fLo /tmp/x/complete.zip https://library.ldraw.org/library/updates/complete.zip && shasum -a 256 /tmp/x/complete.zip
unzip -p /tmp/x/complete.zip ldraw/CAreadme.txt
grep -h '^0 !LICENSE' -r tests/fixtures/ldraw-mini --include='*.dat' | sort | uniq -c
node scripts/ldraw-mini-check.mjs

# LDCad shadow library: license, file list, and coverage of ldraw-mini
gh api repos/RolandMelkert/LDCadShadowLibrary/contents/LICENSE.md -H "Accept: application/vnd.github.raw" | head -1
gh api "repos/RolandMelkert/LDCadShadowLibrary/git/trees/9b1131f?recursive=1" --jq '.tree[].path' > /tmp/shadow.txt
grep -c '^parts/[^/]*\.dat$' /tmp/shadow.txt   # 3,349 part files
grep -c '^p/' /tmp/shadow.txt                  # 87 primitives
# ldraw-mini's real parts (titles not starting ~, =, or _) that have a shadow file: 94 of 141
n=0; for f in $(awk '{print $2}' tests/fixtures/ldraw-mini/MANIFEST.sha256 | grep -E '^parts/[^/]+\.dat$'); do
  case "$(head -1 tests/fixtures/ldraw-mini/$f | tr -d '\r')" in "0 ~"*|"0 ="*|"0 _"*) continue;; esac
  grep -qix "$f" /tmp/shadow.txt && n=$((n+1)); done; echo $n

# The projects: license and commit read
for r in anteloc/ldraw-nova jonx/BrickBuilderMCP ElRashoMacuin24/brickbuilder-mcp datakurre/brick-mcp musharna/ldraw-mcp BaLaurent/ClawdCAD AnyiWang/OpenCovibe; do
  echo "$r $(gh api repos/$r/commits/HEAD --jq .sha[0:7]) $(gh api repos/$r/license --jq .license.spdx_id 2>/dev/null || echo none)"; done

# Rebrickable and the providers
scripts/terms-check
```
