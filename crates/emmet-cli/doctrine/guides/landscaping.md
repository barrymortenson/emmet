# Landscaping
Also: landscape, garden, gardens, terrain, nature, ground, base, diorama

The ground a model stands in: lawns, paths, water, rock, and plants. It turns a building on a baseplate into a scene.

## Rules

- The model covers at most half its base; the landscape gets the rest.
- Ground is never one level. Use three to five heights: water lowest, paths 1 to 2 plates up, lawn 2 to 4 up, banks
  and rocks above, plants on top. Steps between levels are 1 to 3 plates, not bricks (`contour_terraces`,
  `hillside`, `grass_edge`, `sunken_pond_bank`).
- No straight edge longer than 6 to 8 studs: break it with a round-corner plate, a bite, a rock, or a plant
  (`organic_ground_outline`, `grass_edge`). Rectangles read as man-made; that's right for paving and wrong for lawns.
- One dominant ground color and one or two accents covering less than 15% of the area. Lawn green or bright green;
  beds dark green or reddish brown; paths tan, dark tan, or light bluish gray; water light aqua or trans-light blue
  tiles; rock in two grays, never one, never three.
- Paths lead somewhere: from the gate to the door, from the door around to the garden (`curved_garden_path`,
  `dirt_path`, `stepping_stones`, `flagstone_paving`).
- Plants in drifts, not rows: two or three heights together, a tall one behind low ones. A garden leaves 30 to 50% of
  the lawn open; a jungle leaves no stud showing near a wall.

## Pieces

- Boundaries: `fence`, `garden_gate`, `clipped_hedge`, `bush`.
- Beds and plants: `flower_bed`, `potted_plant`, `lawn_with_plant_sockets` (trees and lanterns plug in),
  `bamboo_grove`, `lily_and_reeds`.
- Trees: `tree` (classic), `blossom_tree`, `layered_conifer` or `pine_tree`, `cypress`, `palm_tree`, `gnarled_tree`.
  One tree taller than the building's ridge frames it.
- Water: `pond`, `tiled_pond_water`, `river`, `waterfall`, `surf_line` (a shore).
- Rock: `garden_boulder` (smooth, for gardens), `rock_outcrop`, `rock_facet`, `cliff_wall`, `rock_spire`.
- Props: `bench`, `lamp_post`, `stone_lantern`, `woodpile`, a mailbox at the gate.
- Seasons: `snow_drift_edge`, `snow_cap`, `studs_up_snow_roof`.

## By scale

- Micro: a tree is 1 to 3 parts (a brown 1 x 1 under a green leaf piece or a 2 x 2 round plate), water a trans-blue
  tile, a path a 1-wide tile line.
- Figure: the pieces above.
- Large and botanical: stems are axles with bar tubes, petals are curved parts on clips; search past "plant" parts for
  the shape you need (a fender, a dish, a shield) and say why in a comment.
