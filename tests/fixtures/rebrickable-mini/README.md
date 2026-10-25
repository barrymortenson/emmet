# rebrickable-mini

Part, color, and set data from Rebrickable (rebrickable.com).

A cut of Rebrickable's catalog download of 2026-10-24 (https://rebrickable.com/downloads/), which Rebrickable
offers free for personal and commercial use with credit to Rebrickable as the data source. Built by
`scripts/make-rebrickable-mini` for tests (plan section 12, ADR 0004): the rows for the 185 Rebrickable parts
behind ldraw-mini's parts and the rung 3 shortlist sets (40182-1, 40142-1, 21022-1, 21013-1, 6743-1), all colors and themes, and those sets'
inventories (23 colors). The color and part mappings keep only their LDraw (LD) and BrickLink (BL)
rows, and the `img_url` columns are dropped; the rows kept are otherwise unchanged.

| File | Rows here | Rows in the download |
|------|-----------|----------------------|
| colors.csv | 275 | 275 |
| color_mappings.csv | 438 | 846 |
| part_categories.csv | 76 | 76 |
| parts.csv | 185 | 64906 |
| part_mappings.csv | 394 | 183142 |
| part_relationships.csv | 25 | 37543 |
| elements.csv | 9822 | 114675 |
| themes.csv | 496 | 496 |
| sets.csv | 5 | 28553 |
| inventories.csv | 5 | 47820 |
| inventory_parts.csv | 318 | 1570284 |

Source files (sha256 of each .csv.gz):

- `colors.csv.gz` `8eafe535e3e2d7dd46ee9349335c52c680509016b7fdf83bac67082b4ec9babe`
- `color_mappings.csv.gz` `b6b6fc42f14184caf2a160d71642a225fa37ea64e094419a5fb4e0566a84b7a4`
- `parts.csv.gz` `8c3cd699daab3ca7dac0d71003d0cf81fb9215122741ae40703e41671d15fa59`
- `part_categories.csv.gz` `1d79a237708014aecfd4eadcc5a5db8cb20efe8b9f9d3f105ec2c8d9328cefc6`
- `part_mappings.csv.gz` `1d2ba5431f87a864427f8a1c5c79f80403adcadd6f75189aa50e9999136f60ec`
- `part_relationships.csv.gz` `77197478b8af96a48964ac91b37983fffebdfb63183f2826da64f56bb961d03d`
- `elements.csv.gz` `93a686f4d84516c25936e5d1b2c55511a96a89e6167c0eb0e36064cde86ddce4`
- `sets.csv.gz` `877c7b7b0c86a86cd9d334c96c0932163d948244bc04f7da38dfe90cd2657fed`
- `inventories.csv.gz` `0564a6b8ab3d5fa53ff9029ae84ca43e0f265c2e9a8f909bd17cf230ec3eacb8`
- `inventory_parts.csv.gz` `01c3b8a9b81c06ceaf018485a4f12b0c5639d0799f7116f96f3e3808e03b968b`
- `themes.csv.gz` `fb2cd32e4c52ef1db1fdda53c61e26fe00204612ef9df40e5f38980d391f3053`
