# ADR 0004: LDraw ↔ Rebrickable ↔ BrickLink mapping

Status: accepted, 2026-10-24 (session 0B). Settles Q-03.

## Context

The BOM, availability, and exports need each LDraw part and color as Rebrickable and BrickLink know it (plan section
12). Section 12 named the source: `part_mappings.csv` rows with `ext_system = LD`, plus identical ids. This ADR
measures that on the pinned data and fixes the rules.

Data: the LDraw library 2026-09 (`complete.zip`, sha256 `f3e496d0…a4dd5f2`) and Rebrickable's catalog download of
2026-10-24 (each file's sha256 is in `scripts/make-rebrickable-mini`).

## Decision

### Parts: four rules, in order

An LDraw part id maps to a Rebrickable `part_num` by the first rule that applies:

1. **An LD row** in `part_mappings.csv` whose `ext_id` is the LDraw id (the row marked `is_import_id` wins if there
   are several).
2. **An LD row for a moved id.** LDraw renames parts and leaves a `~Moved to <new id>` file behind; Rebrickable's LD
   rows sometimes still name the old id. Example: Rebrickable's `75541` (9V switch point right) maps to LDraw
   `2859c01`, which the 2026-09 library has moved to `75541-f1`. So the rule follows LDraw's moved files backwards.
3. **An identical id** in `parts.csv`.
4. **The base of a formed variant.** LDraw's flexible parts come in formed variants (`75541-f1`, `75541-f2`); strip the
   `-f<n>` suffix and try again.

The reverse (a Rebrickable inventory line to an LDraw part) takes the LD row marked `is_export_id`, else the
identical id, as long as the library has that file.

A part with no mapping shows "availability unknown", never "unavailable" (section 12).

### Coverage

Measured by `tests/spikes/data/coverage.mjs`:

| Set of parts | Map | How |
|---|---|---|
| ldraw-mini's 141 real parts (its 36 aliases and internal `~` parts resolve to these) | **141 (100%)** | 137 LD row, 4 LD row for a moved id |
| The whole library's real parts (titles not starting `~`, `=`, or `_`) | 12,575 of 20,315 (61.9%) | 12,495 LD row, 70 identical id, 8 moved id, 2 formed variant |
| Every `parts/*.dat` file | 13,233 of 24,856 (53.2%) | |

The unmapped real parts are mostly stickers (4,604) and printed or patterned parts (1,733), then assemblies with a
`cNN` id (520), minifig and figure parts (412), other (433), and Duplo-style parts (38). These are the parts a
generated model rarely uses, so "availability unknown" stays rare in practice.

Three sample sets, Rebrickable inventory to LDraw (spare parts left out), from the rung 3 shortlist:

| Set | Inventory lines | Pieces |
|---|---|---|
| 40182-1 Bricktober Fire Station | 66 of 66 (100%) | 175 of 175 |
| 21022-1 Lincoln Memorial | 36 of 36 (100%) | 274 of 274 |
| 6743-1 Street Speeder | 69 of 69 (100%) | 165 of 165 |

The other two shortlist sets (40142-1, 21013-1) also map 100%.

### Colors: one table from `color_mappings.csv`

- **Source:** `color_mappings.csv`, columns `id` (the Rebrickable color), `ext_system`, `ext_id`, `ext_descr` (a JSON
  list of the external names), `is_export_id`, `is_import_id`. It has 222 LD rows, 216 BL rows, 205 BO rows, and 203
  LE rows.
- **The table Emmet builds** (the `colors` table in section 12's index) has one row per Rebrickable color, with the
  LDraw code and name, the BrickLink id and name, and RGB from `colors.csv` (BO and LE rows are dropped). Every
  spelling becomes an alias, so "Light Bluish Gray", "Light_Bluish_Gray", and LDraw code 71 resolve to the same
  color.
- **Several LDraw codes can map to one Rebrickable color** (3 colors), such as LDraw 256 Rubber_Black (an import-only
  row) and 0 Black, which both map to Rebrickable 0 Black. LDraw → Rebrickable uses any LD row; Rebrickable → LDraw
  uses the `is_export_id` one.
- **Coverage:** 217 of LDConfig.ldr's 322 LDraw colors map to a Rebrickable color (67.4%). The rest are mostly
  Modulex, fluorescent inks, and rare metallics. Of the 272 Rebrickable colors that appear in parts, 213 have an LDraw
  code and 216 a BrickLink id.

The colors the ladder uses, as an example:

| Rebrickable | LDraw | BrickLink |
|---|---|---|
| 19 Tan | 19 Tan | 2 Tan |
| 320 Dark Red | 320 Dark_Red | 59 Dark Red |
| 2 Green | 2 Green | 6 Green |
| 15 White | 15 White | 1 White |
| 0 Black | 0 Black (and 256 Rubber_Black, import only) | 11 Black |
| 71 Light Bluish Gray | 71 Light_Bluish_Gray | 86 Light Bluish Gray |
| 72 Dark Bluish Gray | 72 Dark_Bluish_Gray | 85 Dark Bluish Gray |
| 4 Red | 4 Red | 5 Red |
| 70 Reddish Brown | 70 Reddish_Brown | 88 Reddish Brown |
| 47 Trans-Clear | 47 Trans_Clear | 12 Trans-Clear |
| 288 Dark Green | 288 Dark_Green | 80 Dark Green |
| 28 Dark Tan | 28 Dark_Tan | 69 Dark Tan |
| 14 Yellow | 14 Yellow | 3 Yellow |
| 10 Bright Green | 10 Bright_Green | 36 Bright Green |

Rebrickable's color ids follow LDraw's codes for most solid colors; BrickLink's don't, which is why the BrickLink
XML export needs the table.

### The test fixture

`scripts/make-rebrickable-mini` cuts the pinned download to the rows ldraw-mini's parts and the rung 3 shortlist need:
185 parts, 9,822 elements, all 275 colors, the LD and BL color and part mapping rows, and the five sets' inventories
(318 lines), in `tests/fixtures/rebrickable-mini/` (300 KB) with Rebrickable's credit. The `img_url` columns are
dropped. It applies the same four rules, so 3B's loader and this ADR agree.

## Consequences

- 3B implements the four rules in `emmet-core` with a test per rule (the 9V switch covers rules 2 and 4).
- Rung 2's switch is a 9V switch point (`75541`/`75542`), because the 2026-09 library has no plastic train switch.
  It maps to Rebrickable through rules 2 and 4.

## Rerun

```sh
# The pinned inputs (Rebrickable allows automated downloads at most once a day)
mkdir -p /tmp/rb /tmp/ld
for f in colors color_mappings parts part_categories part_mappings part_relationships elements sets inventories inventory_parts themes; do
  curl -fLo /tmp/rb/$f.csv.gz https://cdn.rebrickable.com/media/downloads/$f.csv.gz; done
curl -fLo /tmp/ld/complete.zip https://library.ldraw.org/library/updates/complete.zip

node tests/spikes/data/coverage.mjs /tmp/rb /tmp/ld/complete.zip 40182-1 21022-1 6743-1 40142-1 21013-1
scripts/make-rebrickable-mini /tmp/rb   # fails unless the files match the 2026-10-24 pin
```

A newer download gives slightly different counts; the 2026-10-24 numbers above are the record.
