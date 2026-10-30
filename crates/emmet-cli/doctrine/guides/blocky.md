# Blocky
Also: voxel, voxels, voxel art, cube, cubes, cubic, block world, blocks

Worlds made of cubes, like a voxel game's: every outline on a grid of blocks, textures from mixed shades, no curves.
When the user names a voxel game, this is the guide for its look; never use the game's name in the model.

## Rules

1. Grid first. Pick the block size and never break it. At figure scale a block is 2 x 2 studs and 5 plates tall (a
   brick and two plates; 4 or 6 will do, the footprint won't). Every block sits on even stud coordinates
   (`block_column`, `grass_block`).
2. Every edge on a block boundary. Terrain changes material, rises, or falls only at block edges: steps, not slopes;
   squares, not curves (`stepped_block_terrain`, `raised_module_base`).
3. No slopes, curves, or round parts in the outline. Roofs climb in 1-stud steps (`stepped_plank_roof`,
   `flat_jumper_roof`); hills terrace; trees are cubes (`block_tree`). Round parts only as small detail inside a block
   or for a thin game object (a torch, a crop).
4. Texture by mixing two or three shades, never by printing: stone light and dark bluish gray (plus green for moss),
   wood tan planks and reddish brown logs, leaves green, dark green, bright green, and trans-green, sand tan, dirt
   reddish brown (`speckled_block_wall`, `mossy_stone_block`, `plank_and_stone_wall`).
5. A single stud on a smooth top reads as one block: smooth tops with tiles and 2 x 2 jumpers, and center thin things
   on a jumper (`flat_jumper_roof`, `stick_torch`, `block_fence_post`).
6. Thin game things are 1 x 1 and centered: torches, fence posts, crops, flowers, reeds (`crop_field`, `reed_stalks`).
7. Creatures are boxes: a box body, box legs, a box head (`block_beast`); figures with cube heads.
8. Palette: bright green, green, dark green, reddish brown, tan, light and dark bluish gray, white, blue and
   trans-light blue for water, orange for lava, lime for reeds. Earthy and saturated; no purple, magenta, or dark tan.
9. Build in modules on legs: each scene a raised slab with a dirt edge, linked to the next by link blocks, so the user
   can lay the world out more than one way (`raised_module_base`, `module_link_block`).
10. Detail is low: a wall is plain texture; a few feature blocks (a bookshelf, a torch, a window, a crafting bench)
    carry the story (`bookshelf_block`, `lava_inset`, `block_water_and_falls`, `cactus_column`).

## Scenes

A blocky model is a scene, not a building on a bare baseplate: the ground is blocks too. Raise it on a module with a
dirt edge, step the terrain a block at a time, scatter grass blocks, run a path of a different material, plant block
trees and crops, and put water or lava in a pit (`raised_module_base`, `stepped_block_terrain`, `grass_block`,
`block_tree`, `crop_field`, `block_water_and_falls`, `lava_inset`). The building covers at most half the base.

## Buildings

At figure scale the houses keep 1-stud walls and a figure's door (1 x 4 x 6); the blocky look comes from plank and
stone textures, square windows, and stepped roofs. A house is about 10 x 12 studs.

## By scale

- Micro: a block is 1 x 1 (a 1 x 1 brick and a plate). A tree is a 1 x 1 brown stack under a 3 x 3 green stack.
- Figure: 2 x 2 blocks, as above. A tree is a 2 x 2 trunk 3 to 5 bricks tall under a 6 x 6 to 8 x 8 canopy two bricks
  deep.
- Large: a block is 4 x 4 studs, 10 plates tall, each face a tiled panel with a 4 x 4 pixel texture of 1 x 1 tiles
  (`pixel_mosaic_panel`).
