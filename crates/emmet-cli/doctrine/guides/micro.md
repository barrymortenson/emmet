# Micro scale
Also: microscale, micro scale, nano, skyline, tiny, miniature

The smallest scale: a skyline, a landmark, a ship or a city at about a plate a story, 1:500 and smaller. Nobody reads
a window here; they read outlines and one texture.

## Rules

- Silhouette first, then one texture. Each building gets one surface texture (grille ribs, log grooves, tile seams,
  a stripe of 1 x 1 bricks) and one crown. Never three textures on one tower (`floor_band_bricks`,
  `tower_side_fins`, `curtain_wall_glazing`).
- One part per feature: a tree, a car, a boat, a pair of columns is one part or two. Use parts for what they aren't:
  a cone as a tree, a dish as a spiral ramp, binoculars as columns (`micro_tree_cluster`, `micro_traffic`,
  `micro_boat`, `spiral_dish_stack`).
- Towers are 1 x 1 to 2 x 3 in plan; a plate is about a story, so a 40-story tower is 12 to 15 bricks and a 100-story
  one about 25. Tall 1 x 1 x 3 and 1 x 1 x 5 bricks in a cluster read as piers (`micro_tower`).
- Stagger heights by at least a brick and lead the eye to one or two landmarks 1.5 to 2 times taller than the rest.
  Every tower gets a crown: a curved top, a pyramid, a gable, a spire (`tower_crown`, `bar_spire`,
  `stepped_tower_setbacks`, `tower_setback_plinth`).
- Depth by color and rows: three rows deep, light landmarks in front, darker blocks behind; the back row may be a
  wall of towers a stud thick (`skyline_depth_rows`).
- A tight palette: four to six colors plus one accent (the yellow taxis, a red roof).
- No figures, no doors, no window frames. A person is never built; a monument may be a whole figure used as a statue
  (`statuette_group`). A crowd is the seats themselves (`raked_seating_panel`).
- Studs mostly hidden: roofs, plazas, and the base are tiled; a few open studs read as rooftop clutter.
- Water is pale tiles with offset seams; roads are dark tile strips between the buildings (`water_tile_field`).

## Presenting it

A base no deeper than it needs (8 to 16 studs), dark and tiled, with a name tile on the front edge; a backdrop, if
any, built flat and stood up (`architecture_base`).

## In a bigger model

Micro houses, towers, and windows on the walls and tiers of a figure-scale castle or city make it read as huge
(`micro_house_on_wall`).

For a micro city on modules, see `guide micropolis`; for larger architecture models, `guide midi`.
