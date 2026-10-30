# Town
Also: city, village, houses, house, cottage, modular, street, shop, shops

Houses, cottages, shops, and streets at figure scale (`guide figure-scale` for the sizes). What makes a building read
as a real one with character, not a box with holes in it.

## Character

- No face is symmetric. One big feature per face (a bay window, a porch, a cross gable, a chimney), never two at the
  same spot, and the front door off center. A town building may be symmetric about its entrance, then breaks it with
  a sign, a lamp, or a balcony on one side.
- Two or three wall textures, never one. The heavier one goes on the lower third: a stone or brick plinth or
  wainscot, then plaster, siding, or timber above (`wainscot_split_wall`, `masonry_wall`, `two_tone_masonry`,
  `plaster_with_exposed_stones`, `clapboard_siding`, `half_timber_band`, `timber_frame_panels`). Contrast corners
  (`quoin_corner`, `rusticated_stone_pilaster`).
- Nothing flush for long. Sills and string courses stand a plate proud, pilasters a plate to a stud, windows and doors
  sit a stud back, signs and awnings stand 1 to 2 studs out. That relief is most of what reads as architecture
  (`string_course`, `window_sill`, `window_hood`, `panel_pilaster`, `dentil_band`, `wall_top_cornice`).
- Steep, overhanging roofs. A cottage roof is 45° and as tall as its walls; it overhangs a stud at the eaves and its
  edge is finished (`eaves_fascia`). A shallow roof reads as modern or Mediterranean.
- A chimney, always taller than the ridge by 1 to 2 bricks, with pots (`chimney_stack`, `banded_chimney`).
- Stories read as stories: a different color or texture per story (`two_story_color_split`), a ledge of rough stone
  under a jettied or set-back story (`rough_stone_ledge`), a balcony on corbels (`corbelled_balcony`). A storybook or
  cartoon house paints its wall in blocks of bright color, each with its own window (`color_block_facade`).
- About one small detail every 2 to 3 studs of face: a lamp, a window box, a downpipe, a sign, a planter
  (`window_box`, `wall_lamp_bracket`, `downpipe`, `hanging_planter`, `blade_sign`).

## Palette

Four or five colors plus one accent. The roof differs from every wall color; window frames and shutters share one
color; the accent shows 3 to 6 times (flowers, a lantern, the door). A town street gives each building, and each story
of a tall one, its own 2 or 3 colors.

- Cottage: white or tan plaster, medium nougat or dark bluish gray stone, dark red or dark bluish gray roof, dark
  green or reddish brown door and shutters.
- Town house: medium nougat or dark orange brick, tan stone trim, white window frames, sand blue or dark red roof.

## Openings

- Door: `door_frame` (a 1 x 4 x 6 door in its frame) or `plank_door`; a `doorstep_and_mat` or steps up to it; a
  `door_hood`, `turned_post_porch`, or `porch_posts` over it.
- Windows: `shuttered_window`, `window_frame`, `bay_window`, `twin_arched_window`, `round_window` (in a gable); dress
  them with `window_sill`, `window_hood`, `eyebrow_arch_hood`, `dome_window_hood`, `window_box`.
- Shops: `shop_window`, `lattice_shopfront`, `round_corner_bay`, `scalloped_awning`, `signboard_beam`.

## Roofs

`gable_roof` is the default; then `hip_roof`, `mansard_module` (a town house's steep top story), `cross_gable` (breaks a
long roof, carries a nameplate), `roof_dormer`, `curved_dormer`, `mini_dormer`, `ridge_cap`, `roof_cresting`,
`studs_up_snow_roof` or `snow_cap` in winter. Roof surfaces: plain slopes at Low detail, `plate_shingle_roof`,
`tile_stripe_shingles`, or `tiled_roof_panel` at High. Seat it with `lift_off_roof_seat` so it lifts off.

## Layout

- A cottage: a rectangle 12 to 16 by 8 to 10 on a 32 x 32 base, raised a brick (`lattice_skirt_foundation` under a
  porch), with the garden as big as the house: a path to the door, a fence and gate, beds, a tree, a bench
  (`guide landscaping`).
- A town building: 16 or 32 wide, about 24 deep, with 6 to 8 studs of sidewalk in front (`sidewalk`, `lamp_post`,
  `bench`). Pilasters 2 studs wide at each end and between bays, a string course at each floor line, a cornice or
  `balustrade_parapet` on top. Stories stack and lift off (`lift_off_floor`).
- Inside, every room says what the building is (`guide interiors`).

## Common mistakes

- One color and one texture for every wall: add a plinth and a second texture.
- Windows with nothing round them: give each a sill and a hood, or shutters, or a box.
- A roof that stops at the wall: overhang it a stud.
- A house in the middle of a bare baseplate: landscape the rest.
