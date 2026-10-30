# Video game
Also: game, video games, retro game, platformer, arcade, kart racer, pixel art, 8-bit, retro console

Bright game worlds and the hardware they ran on: chunky characters, iconic objects, pixel art, and retro consoles.
When the user names a game or a console, build its look with this guide; never use its name, its characters' names,
or a logo in the model, and say in the reply that the model is in that style, not the official thing.

## Rules

1. Chunky proportions. Characters are 2 x 2 studs wide with a head as big as the body, 4 to 5 bricks tall
   (`cube_head_character`, `brick_enemy`). Vehicles are stubby: wheelbase about 1.5 times the width, wheels nearly as
   tall as the body, 4 to 8 studs long (`chunky_kart`).
2. A bright, flat, saturated palette: red, yellow, blue, bright green, white, one accent per object. Little gray, no
   brown except wood and enemies. Two or three colors an object.
3. Iconic objects keep their outline, from the simplest parts: an item block is a 2 x 2 block (`item_block`), a
   spotted cube is white with red round tiles (`polka_dot_cube`), a coin a round tile, a cloud white curved slopes, a
   pipe green round bricks with a wider lip (`warp_pipe`). Floating things float on clear stands (`float_on_clear`).
4. Pixel art is flat tiles, 1 stud a pixel: built flat and stood up for screens and signs, studs up only for floors
   (`pixel_mosaic_panel`, `tile_brick_texture`). Draw a texture with tile seams, not stud patterns.
5. Level pieces: platforms on `bolted_pillar`s, a `flip_bridge`, a `start_gantry` and `curb_rumble_strip` for a race,
   an `a_frame_tent` and `brick_campfire` for a camp.
6. Retro hardware is a studs-out box: tiled on every face, two-tone (light and dark gray, or wood and gray), round tiles
   for knobs, grille tiles and fins for vents, a set-back base (`console_shell`, `crt_cabinet`, `controller_slab`,
   `vent_fins`).
7. One simple, visible play feature an object: a crank, a push rod, a flick launcher, a flip bridge
   (`item_launcher`, `crank_winch`).
8. Hidden structure can be any color; only what shows follows the palette.

## By scale

- Toy scale (brick-built characters): an item block is a 2 x 2 brick; a kart is 4 x 8.
- Figure scale: an item block is a 2 x 2 block on a stand above a figure's head; a pipe is a 2 x 2 round stack with a
  4 x 4 round lip.
- Large: objects become studs-out panels or pixel mosaics; a screen 16 studs tall holds a level whose blocks are 2
  pixels wide.
