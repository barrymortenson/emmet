# Micropolis
Also: micro city, microcity, micro town, city block

The community's micro city standard: a city built from 16 x 16 modules that tile into any layout. A story is about a
brick (9 feet), so whole streets fit on a table.

## The standard

- Each block is a 16 x 16 module on a 16 x 16 base (or four 8 x 8 plates tied with a second layer).
- A 2-stud road of dark bluish gray tiles along each side the block faces a street, so two modules' roads meet as a
  4-stud street. A 1-stud sidewalk of light bluish gray tiles inside the road where it's used. Road corners are tiled,
  never studs. Nothing on the road but cars (`micropolis_block`, `micro_traffic`).
- Buildings stand inside the 12 x 12 middle: one building up to 12 x 12, or several from 4 x 4 to 6 x 12 with 1-stud
  gaps or alleys.
- Keep the road and base colors the same on every module, so modules tile.

## Buildings

- A story is a brick for upper floors and 4 to 5 plates for the ground floor. A house is 2 to 3 stories, a shop block
  3 to 5, an office tower 8 to 20, a landmark up to 40.
- Windows are 1 x 1 or 1 x 2 elements (a trans or headlight brick, a grille tile) set flush or half a plate back; a door
  is a 1 x 1 dark tile or a 1 x 2 gap at street level. No figure-scale windows or doors.
- One texture per building and a crown on the tall ones (`micro_tower`, `floor_band_bricks`, `tower_crown`,
  `tower_setback_plinth`, `stepped_tower_setbacks`, `curtain_wall_glazing`, `paired_colonnade` for a civic front).
- Flat roofs are tiled with a few round plates and grille tiles as rooftop machinery; pitched roofs use 1 x 2 and
  2 x 2 slopes and cheese slopes.

## Street life

Every module has at least one of: a tree (a 1 x 1 round brick under a 2 x 2 round plate or a scoop), a car (a 1 x 2
tile on a jumper), a lamp (a bar in a 1 x 1 round plate) (`micro_tree_cluster`, `micro_traffic`). A park, a harbor
(`water_tile_field`, `micro_boat`), or a sports ground across 2 x 2 modules (`mowed_pitch`, `raked_seating_panel`)
breaks up the buildings.

## Steps

Base, road tiles, the buildings' outlines, a story a step, the roof, then the street details.
