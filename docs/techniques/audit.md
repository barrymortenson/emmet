# The technique audit

Session 3E's audit of every technique in `data/techniques/` against the building rules (`docs/spec/building-rules.md`,
approved by Barry, H-27): for each, how a figure meets it at figure scale, its figure-facing sizes and defaults as the
audit found them, what the caller couldn't change, what it didn't return that the next part needs, the verdict, and
what changed. Fixes are in the technique, never in the doctrine; every changed technique's example, render, and test
were regenerated, and each example validates clean, figure_clearance included (`tests/it/tests/techniques.rs`, which
also checks this table has a row for every technique).

Four readers drafted a row each from the code; the verdicts and every fix were checked against the code and tested.
**331 fine, 39 fixed, 8 limited by design.** Beyond the 4B station's canopy and
porches, the audit found doorways that kept nothing clear or were too low for a figure (`portcullis` was 6 plates,
`cave_mouth` about 10, `brick_built_arch` 3), seats with no room declared, a colonnade 9 plates high, a footbridge whose
posts left 1 stud between them, three vehicles whose seat faced the rear a quarter stud off the grid, and cabs 5 plates
over the seat.

Figure: walk-under, walk-through, sit, walk-on, walk-past, or none. Sizes in studs across and plates up (3 a brick).

| Technique | Figure | Sizes and defaults (as found) | Can't change (as found) | Doesn't return (as found) | Verdict | What changed |
|---|---|---|---|---|---|---|
| `a_frame_tent` | sit | a bed a figure lies in: floor plates at z, 6 deep (DEPTH) x length 6 (4 to 10); sleeping bag (2 x 4 plate at z+1, 2 x 2 tile, 15068 pillow to z+4) in rows y+2..y+4, bag top z+3; walls 1 brick (z+1..z+4) on rows y and y+5; canvas hinged from z+4 up to the … | DEPTH = 6; walls one brick tall, so the canvas starts at z+4 | returns the ridge top only; not the bag's top (z+3) or footprint | limited: a figure lies in it, entered from the open ends; rule 3's seat spaces don't fit | — |
| `angled_display_stand` | none | — | — | — | fine | — |
| `angled_hull_panel` | none | — | — | — | fine | — |
| `antenna_mast` | none | — | — | — | fine | — |
| `architecture_base` | none | — | — | — | fine | — |
| `armchair_and_sofa` | sit | armchair 4 x 2, sofa 6 or 8 x 2; seat (cushion tiles) top 3 plates; the seat between the arms is width - 2 studs (2 for the armchair) and 1 deep (row 0); arms a 1 x 2 brick + 11477 up to 6 plates at each end; back 2 bricks, capped at 8 plates; back pads … | seat 3 plates high and 1 deep; the arms take the end studs (by design) | returns the Handle only; no seat top | fixed | keeps 2 studs of legroom and 9 plates over the seat clear (the arms are its ends) |
| `arrow_slit` | none | — | — | — | fine | — |
| `axle_round_stack` | none | — | — | — | fine | — |
| `ball_joint_arm` | none | — | — | — | fine | — |
| `balustrade_parapet` | walk-past | a railing on a roof edge or a terrace: base plate z, 1 x 1 balusters z+1..z+4, handrail plate z+4, tiles to z+6 (returned); 1 stud deep | — | — | fine | — |
| `bamboo_grove` | none | — | — | — | fine | — |
| `banded_chimney` | none | — | — | — | fine | — |
| `banded_round_turret` | none | — | — | — | fine | — |
| `bar_counter` | walk-past | counter 2 deep x 2 x panels (6 by default) long; top plate at z+7, tiles at z+8 (top surface z+9); barkeep's side behind | — | — | fine | — |
| `bar_spire` | none | — | — | — | fine | — |
| `bar_truss_brace` | none | — | — | — | fine | — |
| `bare_stud_greeble` | none | — | — | — | fine | — |
| `bartizan` | none | — | — | — | fine | — |
| `baseplate_road` | walk-on | road width 9 (3 or more), surface z+1 | — | — | fine | — |
| `battlements` | walk-on | wall walk 1 stud wide (row y+1), returned z (z+1 with the tie); breastwork 1 brick + merlons 1 brick + cap, so the parapet stands about 7 plates over the walk | — | returns the walk's z, as a figure needs | fine | — |
| `bay_window` | none | — | — | — | fine | — |
| `beam_spine_floor` | none | — | — | — | fine | — |
| `belfry_louvers` | none | — | — | — | fine | — |
| `bench` | sit | 4 x 2, seat top 5 plates; the example's two benches stood front to front, each in the other's legroom | — | the room a seated figure needs (no keep-clear space) | fixed | keeps a seated figure's room clear (a stud at each end, 2 in front, 3 bricks over the seat); the example's benches stand back to back, not crowding each other's legroom |
| `bermed_wall_reveal` | walk-through | a hill front for a door: wall rows=3 by default (9 plates, z..z+9); beam plate at z+3*rows; turf 2 deep from y-1 at z+3*rows+1, overhanging the face 1 stud; cheeks reach 2*rows studs out each end in rows y..y+2 | the turf always overhangs 1 stud at the wall's top; the wall has no opening (wall=True) | returns the turf's top only; takes no door top | fixed | rows 6 by default (was 3), reaching a 1 x 4 x 6 door's top so the turf clears its way; lower courses split into steps |
| `big_round_eye` | none | — | — | — | fine | — |
| `blade_sign` | walk-past | a 5-brick clip column (z..z+15) in the wall; the sign, a 2 x 6 plate on edge, 15 plates tall from z and standing about 2 studs out from the wall face (SIGN_AT = (-0.2, -3)) | the sign spans the column's whole height from z (no lift above the column's base) | returns the column's top only; not the sign's footprint | limited: the caller sets z; at street level it hangs over a door's way through, so its docstring's z is the caller's to keep at or above the door top | — |
| `block_beast` | none | — | — | — | fine | — |
| `block_column` | none | — | — | — | fine | — |
| `block_fence_post` | walk-past | posts every 2 studs (one per 2 x 2 block), 9 plates tall, linked by rails | — | — | fine | — |
| `block_tree` | none | — | — | — | fine | — |
| `block_water_and_falls` | none | — | — | — | fine | — |
| `blossom_tree` | none | — | — | — | fine | — |
| `bolted_pillar` | walk-on | a game pillar, levels=4 x 4 plates, topped by a 4 x 4 round platform to land on (returned top) | — | — | fine | — |
| `bookcase` | walk-past | width 4 x 2 deep, 6 plates a shelf, 20 plates for 3 shelves; stands against a wall | — | — | fine | — |
| `bookshelf_block` | none | — | — | — | fine | — |
| `bracket_side_panel` | none | — | — | — | fine | — |
| `brick_built_arch` | walk-through | an archway through a 1-stud wall: opening 2 (default) or 4 studs wide; sides rise opening_rows=1 row (3 plates), then the arch (3659, a brick, over 2; 3307, two bricks, over 4); wall rows default opening_rows + arch rows + 1 = 3 (9 plates) or 4 (12) | the opening's clear height at its sides is only 3*opening_rows plates; the crown adds less than the arch's own height (the arches' inner heights not measured) | doesn't return the opening's top or footprint | fixed | opening 4 and opening_rows 5 by default (were 2 and 1); keeps the way through clear when a figure fits; returns .door_top and .door |
| `brick_campfire` | none | — | — | — | fine | — |
| `brick_enemy` | none | — | — | — | fine | — |
| `brick_letters` | none | — | — | — | fine | — |
| `bush` | none | — | — | — | fine | — |
| `buttress` | none | — | — | — | fine | — |
| `cab_over_sleeper` | none | — | — | — | fine | — |
| `cab_seat` | sit | vehicle seat 4079 (2 x 2) at x+2..x+4, its bottom at z, seat top z+1, facing -x; steering wheel holder 30640 (2 x 2) at x..x+2 directly in front, wheel 30663 on its column leaning back (+x) | the wheel holder sits 0 studs in front of the seat (x..x+2) | returns nothing: no seat top, and no z a cab roof must clear (z+10) | fixed | the seat faces the front (it faced the rear, a quarter stud off the grid); keeps 9 plates over it clear; returns the seat's top with .headroom |
| `cactus_column` | none | — | — | — | fine | — |
| `cannon_port` | none | — | — | — | fine | — |
| `canopy` | walk-under | columns a Brick 1 x 1 x 5 (15 plates): the beams' underside at z+15, under an 18-plate door top beside it; spacing 8 | the column height (COLUMN = 15, `canopy.py:33`); a spacing under 4 or a last bay a stud wide | where its columns stand, so benches were placed blind | fixed | height (bricks clear under the beams, 6 by default, was a fixed 5) and spacing 4 or more; no last bay under 3 studs clear; returns .columns and .underside |
| `canopy_hump` | none | — | — | — | fine | — |
| `car_wheels` | none | — | — | — | fine | — |
| `cargo_pallet` | none | — | — | — | fine | — |
| `cave_mouth` | walk-through | opening 4 wide (x+2..x+6): jambs 2 courses (z..z+6), then the arch 15254 (1 x 6 x 2) from z+6 to z+12; the docstring gives the opening as 'a little over 3 bricks' (about 10 plates); the dark back wall 2 studs in (y+2), so the mouth is 2 deep; 75-degree slopes … | all hard-coded: two jamb courses set the opening's height; no argument for it | returns the top only; not the opening's top | fixed | jambs (5 by default; the mouth was about 10 plates high, now 15); keeps the front clear; returns .door_top and .door |
| `chamfered_door_portal` | walk-through | opening 4 wide (x+2..x+6) in an 8-long wall 2 thick (y..y+2); height=5 bricks (15 plates; 3 to 8); 3039 slopes at the foot (z..z+3) and 3660b at the head (z+12..z+15) take a stud each side, so the full 4 studs are clear z+3..z+12 and the middle 2 up to z+15 … | opening width 4 and the chamfer pieces | returns the lintel's top; not the opening's top (door_top) or footprint | fixed | height 6 by default (was 5), lining up with a 1 x 4 x 6 door; keeps the way through its 2-thick wall clear; returns .door_top and .door |
| `checker_light_wall` | none | — | — | — | fine | — |
| `chimney_stack` | none | — | — | — | fine | — |
| `chunky_kart` | sit | seat 4079 at x+2..x+4, bottom z+7, seat top z+8, facing +x; steering wheel 30663 at x+4 right in front; open above (fenders at z+10 cover x..x+2 and x+6..x+8, not the seat) | the wheel at x+4 against the seat | returns the seat's base (z+7) | limited: a game kart; the wheel fills the legroom by design and nothing stands over the seat | — |
| `clamshell_pod` | sit | 8 x 8 round floor at z; seat 4079 at (x+3, y+1), bottom z+1, seat top z+2, facing -y out the open front; back-half shell 2 courses (z+1..z+7); cap plate at z+7 closed (lifted=False, the default) or z+16 lifted, an 8-plate dome on it | SHELL = 6 sets the closed cap at z+7; the lifted post (2454b) adds exactly 9 plates | returns the cap's top only; not the seat's top | fixed | closed, the cap is 9 plates over the seat (was 5); keeps the seat's room clear; returns .underside and .seat |
| `clapboard_siding` | none | — | — | — | fine | — |
| `cliff_wall` | none | — | — | — | fine | — |
| `clipped_hedge` | walk-past | a hedge 1 stud deep, about 8 plates tall, along a wall or path | — | — | fine | — |
| `cockpit_canopy` | sit | a cockpit 4 x 4 from (x, y) on a deck at z; wall behind at y+4, 3 bricks (z..z+9); canopy 2620 hinged at z+8.5, shut it rests on the deck and reaches about 9 plates (CANOPY_BODY 72 LDU); the seat is the caller's | hinge height z+8.5 set by the wall (two 1 x 4 bricks and a 30414) | returns the wall's top (z+9) only | limited: a vehicle cockpit; a shut canopy sits close over the pilot by design, and angle opens it | — |
| `color_block_facade` | none | — | — | — | fine | — |
| `console_shell` | none | — | — | — | fine | — |
| `contour_slab` | none | — | — | — | fine | — |
| `contour_terraces` | none | — | — | — | fine | — |
| `control_console` | walk-past | a desk 2 deep, length 4; rows=1 brick then 45-degree slopes, top at z+6 | — | — | fine | — |
| `controller_slab` | none | — | — | — | fine | — |
| `corbelled_balcony` | walk-under | slab at z, 3 deep from y-2 (2 studs out from the face), length 5 (odd, 3 or more); balusters 3062b on the front row y-2 (z+1..z+4), handrail z+4..z+5; standing room behind the balusters 1 stud deep (y-1..y); corbels 2449 at x, the middle, and x+length-1, 2 … | corbel 2449 hard-coded: hangs 3 bricks (9 plates) under the slab | returns the tie's top (z+2); not the corbels' bottom (z-9), the underside a walk below must clear | fixed | returns .underside (the corbels' bottom, 9 plates under z) and says it clears a door's top; built as a submodel (its corbels went in from below) |
| `countershade_belly` | none | — | — | — | fine | — |
| `crank_winch` | none | — | — | — | fine | — |
| `crop_field` | none | — | — | — | fine | — |
| `cross_display_stand` | walk-past | a 6 x 6 plus-shaped table, top 8 plates; stands in a shop aisle | — | — | fine | — |
| `cross_gable` | walk-under | a cross gable 4 wide at `at`, projecting project=2 studs in front of the wall; soffit plate 3020 at z-1 (y-2..y); nameplate 30414 + tile hanging at z-4..z-1 (project 2) or z-3..z (project 1), 2 studs out | the nameplate hangs 3 plates under the projection (below = zk - 1, the brick at below - 3) | returns the ridge's top only; not the lowest point under the projection (z-4 with the nameplate), which must clear the door's top | fixed | returns .underside and says z is 4 plates or more over a door's top; the example has the door it promised |
| `crows_nest` | walk-on | 8 x 8 round platform, top z+1; rail 48092 one brick (z+1..z+4); a lookout ring 2 studs wide around the 2 x 2 mast | — | returns the platform's top | fine | — |
| `crt_cabinet` | none | — | — | — | fine | — |
| `cube_head_character` | none | — | — | — | fine | — |
| `curb_rumble_strip` | none | — | — | — | fine | — |
| `curtain_wall_glazing` | none | — | — | — | fine | — |
| `curved_booth` | sit | booth 6 x 4; U of seat 1 stud deep, top 3 plates: side seats at x 1..2 and 4..5 (y 0..3), back seat x 2..4 at y 2..3; back 2 bricks + cap (top 8); padding tiles at y 2.6..3, z 4.5..7; table 2 x 2 at (2, 0), pedestal 3941 (2 x 2 round brick), top 6 plates | the pedestal is a 2 x 2 round brick filling the space in front of every seat | returns the Handle only | fixed | keeps 9 plates over its seats clear (the table is where the legs go, by design) |
| `curved_dormer` | none | — | — | — | fine | — |
| `curved_garden_path` | walk-on | a path 2 studs wide, surface z+1 | — | — | fine | — |
| `curved_hood` | none | — | — | — | fine | — |
| `curved_slope_skin` | none | — | — | — | fine | — |
| `curved_wall_end` | none | — | — | — | fine | — |
| `curved_wedge_nose` | none | — | — | — | fine | — |
| `cypress` | none | — | — | — | fine | — |
| `dashboard_and_wheel` | none | — | — | — | fine | — |
| `davit_lifeboat` | none | — | — | — | fine | — |
| `dentil_band` | none | — | — | — | fine | — |
| `dirt_path` | walk-on | a path 2 studs wide, surface z+1 | — | — | fine | — |
| `display_case_counter` | walk-past | counter 2 deep x length 4; top plate z+7, tiles at z+8 (surface z+9) | — | — | fine | — |
| `dock_pier` | walk-on | deck width 4 (2 to 8), planks' top z + 3*height + 3 (z+9 by default); piles under it | — | returns the planks' top | fine | — |
| `dome_from_quarters` | none | — | — | — | fine | — |
| `dome_window_hood` | none | — | — | — | fine | — |
| `door_frame` | walk-through | a 1 x 4 x 6 door in its frame, sill row `sill`, top 18 plates over it | — | the door's top, and its way through (no keep-clear space) | fixed | keeps its door's way through clear on both sides (3 studs, up to its top); returns .door_top and .door |
| `door_hood` | walk-under | 2 studs out at the door's top z; the example placed its door frame by hand with no keep-clear space | — | its underside | fixed | says z is the door's top; returns .underside; the example builds its door with door_frame |
| `doorstep_and_mat` | walk-on | landing a brick (z-3 to z) 2 studs out from the wall, mat tile z to z+1 (under the approach, which starts at z+1), curved-slope step 2 more studs out at z-3 to z-1; pot at x-1 and 1x2 tile at x+width, beside the door's width | — | — | fixed | the example declares its hand-placed door's way through |
| `double_arch_colonnade` | walk-under | plinth 2 plates (z to z+2) runs the full length, also across each opening; columns 3062b height=3 bricks = 9 plates (z+2 to z+11); capital plate z+11; arch 15254 (6 plates) springs at z+12, cornice top z+21; bays 6 studs, 4 studs clear between a bay's columns … | bay width 6 (arch 1 x 6 x 2) and the 2-plate plinth across each opening are fixed; height range capped at 2 to 6 bricks | returns only the cornice top; doesn't return the columns' (x, y) or the openings' underside (za) | fixed | height 6 by default (was 3: arches 9 plates over the plinth), 5 to 8; keeps each bay's way through clear; returns .columns and .underside |
| `downpipe` | none | — | — | — | fine | — |
| `drawbridge` | walk-on | deck width=4 (2 to 8 even), length=6; hinge plates at z, deck plate z+1, planks z+2, deck top z+3, so the deck stands 3 plates over the threshold's studs | — | — | fine | — |
| `dresser_shelf` | none | — | — | — | fine | — |
| `eaves_fascia` | none | — | — | — | fine | — |
| `engine_bay_greebles` | none | — | — | — | fine | — |
| `engine_nozzle` | none | — | — | — | fine | — |
| `exhaust_stack` | none | — | — | — | fine | — |
| `exposed_stud_stonework` | none | — | — | — | fine | — |
| `eyebrow_arch_hood` | none | — | — | — | fine | — |
| `feather_shingles` | none | — | — | — | fine | — |
| `fence` | walk-past | lattice/spindled panels 6 plates (+1 base plate = 7), low 3 (+1); posts 3062b stacked to the panel height; one stud deep | — | — | fine | — |
| `fender_crown` | none | — | — | — | fine | — |
| `figure_bed` | sit | 2 x 6; legs plate z, frame plate z+1, quilt tiles z+2, lying surface z+3; pillow 15068 at y 3 to 5 (z+2 to z+4); headboard 3004 + tile at the head row y=5 up to z+6; posts=True: foot posts on both studs of the foot row (to z+6), head posts to z+9 | the lying length is fixed at 5 studs (quilt + pillow); with posts=True the foot posts take the foot row, leaving 4 studs (unsure a lying figure, about 4.8 studs, fits) | returns the Handle; nothing about where a figure lies | fixed | keeps 6 plates over the quilt and pillow clear; the four-poster is 2 x 8 so its foot posts stand past a lying figure's feet |
| `fin_merlon_crenels` | walk-on | the walk is the back row (1 stud deep) tiled at z (z+1 with tie); breastwork 1 brick on the front row; fins rise rise=1 brick more, 2 studs wide with 2-stud crenels between | — | — | fine | — |
| `flagstone_paving` | walk-on | one tile layer, top z+1; jumpers and bare studs as bumps (bumps=0.08) | — | — | fine | — |
| `flared_exhaust` | none | — | — | — | fine | — |
| `flat_jumper_roof` | none | — | — | — | fine | — |
| `flip_bridge` | walk-on | bridge 4 studs wide (y to y+4), span=4 between pillars levels=3 bricks; bridge top z+3*levels+5 | — | — | fine | — |
| `float_on_clear` | none | — | — | — | fine | — |
| `floor_band_bricks` | none | — | — | — | fine | — |
| `floor_lamp_and_candle` | walk-past | floor lamp 2 x 2, 11.5 plates tall; candle 1 x 1 (furniture-top) | — | — | fine | — |
| `floor_rug` | walk-on | one tile thick, top z+1 | — | — | fine | — |
| `floppy_ear` | none | — | — | — | fine | — |
| `flower_bed` | none | — | — | — | fine | — |
| `flying_buttress_arch` | none | — | — | — | fine | — |
| `footbridge` | walk-on | deck width=3 (2 to 4), span=6; stringers z+3, planks z+4, deck top z+5; posts 3062b 2 bricks (z+5 to z+11) on the end planks at x and x+width-1; handrail 1 x (span+2) plates at z+11 the full length on each side | posts and handrails stand on the deck's outer columns (sides = (x, x+width-1)), and width is capped at 4 by FOOTING/TILE/PLATE | doesn't return the posts' positions or the clear width | fixed | posts stand on the banks outside the deck, so the whole deck is clear between the rails; width 3 or 4; keeps the deck clear; returns .posts |
| `fountain_court` | none | — | — | — | fine | — |
| `four_wide_car` | sit | seat 4079 at (x+8, y+1, z+6), its seat surface z+7, facing -x; windscreen 3823 at x+6 to x+8 from z+6, roof plate at z+12: 5 plates over the seat surface; cab sides 3004 at y and y+3 (two courses, z+6 to z+12) | windscreen and roof height hard-coded at deck+1 (z+6) and the roof at z+12; cab sides 2 courses | returns only the roof top; not the seat's position | fixed | tall=True by default: a course under the windscreen and the roof puts the roof 9 plates over the seat (was 5); the seat faces the front; returns .underside and .seat |
| `front_bumper` | none | — | — | — | fine | — |
| `front_grille` | none | — | — | — | fine | — |
| `front_hinged_hood` | none | — | — | — | fine | — |
| `gable_roof` | none | — | — | — | fine | — |
| `game_pipe` | none | — | — | — | fine | — |
| `garden_boulder` | none | — | — | — | fine | — |
| `garden_gate` | walk-through | opening 4 studs between the posts (x+4*left+1 to x+4*left+5, default x+5 to x+9); gate 3186 on base 3187, 2 bricks; posts 1 x 1 bricks to z+6 + round plate + round tile, top z+8; open=True swings the leaf a quarter turn to the front, standing at x+gate+3, y-3 … | — | returns only the posts' top; doesn't return the opening's position (where a path goes) | fixed | keeps 5 bricks clear through the gate on both sides (3 wide in front when it stands open); returns .door_top and .door |
| `gear_pair` | none | — | — | — | fine | — |
| `glass_box_cab` | sit | 4 x 6 plate z to z+1; seat 4079 at (2,1,1) in the cab's frame, its seat surface z+2, facing -x; 60581 panels 3 bricks z+1 to z+10; roof at z+10: 8 plates over the seat surface; 1 stud in front of the seat (front panel at x 0 to 1); side panels at y 0 and 3 … | the 1 x 4 x 3 panels fix the cab at 3 bricks and 4 x 6; the seat's spot (2,1) is fixed | returns the Handle; not the seat's position | limited: an all-glass box of fixed 3-brick panels has no room for legroom or free ends | — |
| `glow_trim_edge` | walk-on | platform 3 plates thick, top z+3 | — | — | fine | — |
| `gnarled_tree` | none | — | — | — | fine | — |
| `grass_block` | none | — | — | — | fine | — |
| `grass_edge` | none | — | — | — | fine | — |
| `greeble_strip` | none | — | — | — | fine | — |
| `half_roof_hinge` | none | — | — | — | fine | — |
| `half_timber_band` | none | — | — | — | fine | — |
| `hang_on_side_panel` | none | — | — | — | fine | — |
| `hangar_chevron_floor` | walk-on | plates + tiles, top z+2 | — | — | fine | — |
| `hanging_planter` | walk-past | arms 1 x 3 plates reach 2 studs out from the facade at the caller's z; pot + leaves + bloom about 5 plates over the arm | — | — | fine | — |
| `hatch_on_hinge` | walk-on | opening width=2 x depth=2; lid top shut at z+5 | — | — | fine | — |
| `headlights` | none | — | — | — | fine | — |
| `hearth_fireplace` | none | — | — | — | fine | — |
| `hillside` | walk-on | terraces 2 plates high, tread=2 studs deep | — | — | fine | — |
| `hinge_roof` | none | — | — | — | fine | — |
| `hinged_limb` | none | — | — | — | fine | — |
| `hinged_tailgate` | none | — | — | — | fine | — |
| `hip_roof` | none | — | — | — | fine | — |
| `hover_stand` | none | — | — | — | fine | — |
| `interior_arch_doorway` | walk-through | doorway 4 studs wide at gap=2, open opening=5 courses = 15 plates (z to z+15); Arch 1 x 6 Raised (92950, 1 brick) on course 5 at z+15 with its legs on the wall; rows=7 (21 plates) | doorway width fixed at 4 studs | returns the placed Handle only; doesn't return the doorway's top (z+3*opening) or its footprint | fixed | keeps the way through clear on both sides; the Handle carries .door_top and .door |
| `inverted_arch_wheel_well` | none | — | — | — | fine | — |
| `irregular_stone_wall` | none | — | — | — | fine | — |
| `item_block` | none | — | — | — | fine | — |
| `item_launcher` | none | — | — | — | fine | — |
| `jettied_floor` | none | — | — | — | fine | — |
| `jumper_offset` | none | — | — | — | fine | — |
| `kitchen_counter` | none | — | — | — | fine | — |
| `lamp_post` | walk-past | 1 x 1; foot 3062b (3 plates) + post height=5 bricks (2453b) + cone = 21 plates (7 bricks) | — | — | fine | — |
| `lancet_window` | none | — | — | — | fine | — |
| `landing_leg` | none | — | — | — | fine | — |
| `landing_zone` | walk-on | path ring of tiles 2 studs wide level with the seat, top z+1 | — | — | fine | — |
| `large_display_wheel` | none | — | — | — | fine | — |
| `lattice_shopfront` | none | — | — | — | fine | — |
| `lattice_skirt_foundation` | walk-on | raised floor, top z+4 (a brick and a plate) | — | — | fine | — |
| `lattice_tower` | none | — | — | — | fine | — |
| `lava_inset` | walk-on | stone rim of 2 x 2 bricks + jumpers, top z+4; pit a plate below | — | — | fine | — |
| `lawn_with_plant_sockets` | walk-on | lawn a plate thick, top z+1 | — | — | fine | — |
| `layered_conifer` | none | — | — | — | fine | — |
| `layered_plate_cap` | none | — | — | — | fine | — |
| `lean_to_awning` | walk-under | hung at the caller's z and tipped down 20 degrees, so its front edge comes lower than z; length 4 left the posts 2 studs apart | — | how low its front edge comes | fixed | refuses a front edge under 5 bricks over the ground, or under the door top passed as clears, naming the z to hang it at; length 5 or more (posts 3 studs apart); posts a step each; returns .underside and .posts |
| `lift_arm_linkage` | none | — | — | — | fine | — |
| `lift_off_floor` | walk-on | two plate layers; stair opening (2, 4) at the back-right corner, 2 studs wide (a figure's width exactly) | — | — | fine | — |
| `lift_off_roof_seat` | none | — | — | — | fine | — |
| `lift_off_window_cab` | sit | box 4 x 6: frame plates z to z+1, windows 3 bricks z+1 to z+10, roof underside z+10; inside 2 studs deep (x 1 to 3) by 4 wide; the seat is the caller's (example: 4079 on the floor at the cab's z, surface z+1, 9 plates under the roof) | the 1 x 4 x 3 train windows fix the box at 4 x 6 x 3 bricks | — | limited: a cab 2 studs deep inside can't give 2 studs of legroom; the 9 plates of headroom are right | — |
| `liftarm_ladder_frame` | none | — | — | — | fine | — |
| `light_bar` | none | — | — | — | fine | — |
| `lily_and_reeds` | none | — | — | — | fine | — |
| `lit_window_panel` | none | — | — | — | fine | — |
| `loft_half_floor` | walk-under | loft underside at the caller's z (docstring: 5 to 6 bricks over the room's floor; example z=16, 15 plates over the floor's top); two plate layers, top z+2; lattice rail 3633 (3 plates) along the whole open edge from x+1 to x+length-1 | the rail runs the whole open edge with no gap for a ladder (unsure whether one is wanted) | returns z+2 only | fixed (limited: it isn't told where the floor is, so it can't refuse a low z) | says z is the underside, 5 bricks or more over the floor; returns .underside |
| `machicolation` | none | — | — | — | fine | — |
| `mansard_module` | none | — | — | — | fine | — |
| `masonry_wall` | none | — | — | — | fine | — |
| `mast_and_yard` | none | — | — | — | fine | — |
| `micro_boat` | none | — | — | — | fine | — |
| `micro_house_on_wall` | none | — | — | — | fine | — |
| `micro_tower` | none | — | — | — | fine | — |
| `micro_traffic` | none | — | — | — | fine | — |
| `micro_tree_cluster` | none | — | — | — | fine | — |
| `micropolis_block` | none | — | — | — | fine | — |
| `mini_dormer` | none | — | — | — | fine | — |
| `module_link_block` | none | — | — | — | fine | — |
| `mossy_stone_block` | none | — | — | — | fine | — |
| `mowed_pitch` | none | — | — | — | fine | — |
| `muzzle_and_nose` | none | — | — | — | fine | — |
| `oars` | none | — | — | — | fine | — |
| `offroad_fender_flare` | none | — | — | — | fine | — |
| `organic_ground_outline` | none | — | — | — | fine | — |
| `overlap_tile_roof` | none | — | — | — | fine | — |
| `paired_colonnade` | none | — | — | — | fine | — |
| `palm_tree` | none | — | — | — | fine | — |
| `panel_garage_door` | walk-through | opening 8 studs wide (x to x + 8) and 20 plates tall (z to z + 20): jambs of six 1 x 1 bricks and two plates at x - 1 and x + 8; lintel bottom at z + 20, top at z + 23; leaves stand in front of the wall face, 0.8 stud out (OUT) | opening size hard-coded: 8 studs wide and 20 plates tall (range(6) jamb rows + 2 plates, lintel at z + 20); the leaves are fixed 4 x 8 plates (3035), so the width can't change | returns only the lintel's top (z + 23); not the opening's top (z + 20, the line a canopy or the door tops beside it meet) or the opening's footprint | fixed | keeps the space behind the opening clear (a car parks in front); returns .door_top and .door |
| `panel_pilaster` | none | — | — | — | fine | — |
| `pine_tree` | none | — | — | — | fine | — |
| `pinnacle_stack` | none | — | — | — | fine | — |
| `pipe_column_cluster` | none | — | — | — | fine | — |
| `pixel_mosaic_panel` | none | — | — | — | fine | — |
| `plank_and_stone_wall` | none | — | — | — | fine | — |
| `plank_deck` | walk-on | deck top at z + 5 (beam bricks 3 plates, a plate, a tile); length 4 to 16 along x, width 2 to 8; bare 1 x 1 spots about 5% (bare=0.05) | — | returns only the deck's top z, which is all the next part needs | fine | — |
| `plank_door` | walk-through | doorway 4 studs wide (x to x + 4), 15 plates (5 bricks) tall from z; lintel (Brick 1 x 6) bottom at z + 15, top at z + 18; the leaf hangs in front of the wall, from y - 0.8 to y (DOOR_AT) | doorway height hard-coded at 5 bricks: range(5) jamb rows and a Plate 4 x 6 leaf (3032) stood up, 15 plates tall; width fixed at 4 | returns only the lintel's top (z + 18); not .door_top (z + 15) or .door (the footprint), which door_frame returns | fixed | keeps the way through clear on both sides (the front space starts in front of the leaf); returns .door_top and .door |
| `plank_floor` | walk-on | floor top at z + 1 (tiles only), any length x depth | — | returns z + 1, all the next part needs | fine | — |
| `plaster_with_exposed_stones` | none | — | — | — | fine | — |
| `plate_shingle_roof` | none | — | — | — | fine | — |
| `platform_edge` | walk-on | 2-stud band of tiles, top at z + 1; the docstring puts the platform 4 plates or more above the track's bottom | — | returns z + 1, all the next part needs | fine | — |
| `pleated_seat` | none | — | — | — | fine | — |
| `pointed_buttress_strip` | none | — | — | — | fine | — |
| `polka_dot_cube` | none | — | — | — | fine | — |
| `pond` | none | — | — | — | fine | — |
| `porch_posts` | walk-under | posts a round brick and a Brick 1 x 1 x 3: the beam at 12 plates, under an 18-plate door top; a balustrade the whole length | the post height; a way in through the balustrade | the posts' positions and the beam's underside | fixed | height (6 bricks by default; the beam was at 4 bricks and a plate); gap=(offset, width) leaves the balustrade out as the way in, which it keeps clear; returns .posts and .underside |
| `portcullis` | walk-through | wall 3 studs thick (rows y, y + 1, y + 2); gateway width - 2 studs wide (default width=4: 2 studs; width=8: 6), 2 x grilles bricks tall to the arch's springing (default grilles=1: 6 plates; grilles=2: 12 plates), under an Arch 1 x 4 (1 row) or Arch 1 x 8 x 2 … | gateway height capped by grilles in (1, 2): at most 12 plates (4 bricks) to the arch's springing, plus what the arch's curve adds (not computed in the code, under a brick for 3659, about 1 to 1.5 bricks for 3308a, unsure); width only 4 or 8 (ARCHES) | returns only the wall's top; not the gateway's top or its footprint (x + o + 1 to x + o + width - 1, y to y + 3) | fixed | length 10, width 8, grilles 3 by default (a gateway 6 wide and 18 plates tall; was 6 plates); keeps the way through clear; grilles go in a lattice a step; returns .door_top and .door |
| `porthole_band` | none | — | — | — | fine | — |
| `portholes` | none | — | — | — | fine | — |
| `posable_neck_ball` | none | — | — | — | fine | — |
| `potted_plant` | none | — | — | — | fine | — |
| `pyramid_pedestal` | none | — | — | — | fine | — |
| `pyramid_spire_tower` | none | — | — | — | fine | — |
| `quarter_tile_headlights` | none | — | — | — | fine | — |
| `quoin_corner` | none | — | — | — | fine | — |
| `racing_scale_base` | sit | car's front to -x; floor top z + 4 (returned); seats at (rear - 4, y + 2) and (rear - 4, y + 4), each a Plate 2 x 2 at z + 4 with the 4079 seat on it at z + 5, its seat surface at z + 6 (4079 is 8 LDU from its bottom to its seat), facing -x; default wheelbase … | seats' position hard-coded (rear - 4, side by side at y + 2 and y + 4, touching); the seat part and its riser plate are fixed | returns only the cabin floor's top; not the seats' positions or the cabin's span (front + 3 to rear - 1), which a windscreen, a roof, or a steering wheel needs to fit | fixed | the seats face the front; keeps the pair's ends and headroom clear; returns .seats and .cabin |
| `radar_dish` | none | — | — | — | fine | — |
| `radial_spoke_saucer` | none | — | — | — | fine | — |
| `raised_module_base` | none | — | — | — | fine | — |
| `raked_seating_panel` | none | — | — | — | fine | — |
| `rear_spoiler` | none | — | — | — | fine | — |
| `recessed_glow_panel` | none | — | — | — | fine | — |
| `reed_stalks` | none | — | — | — | fine | — |
| `removable_soft_top` | none | — | — | — | fine | — |
| `ribbed_hip_roof` | none | — | — | — | fine | — |
| `ribbed_side_panel` | none | — | — | — | fine | — |
| `ridge_cap` | none | — | — | — | fine | — |
| `ring_light_grille` | none | — | — | — | fine | — |
| `river` | none | — | — | — | fine | — |
| `rock_facet` | none | — | — | — | fine | — |
| `rock_mosaic_panel` | none | — | — | — | fine | — |
| `rock_outcrop` | none | — | — | — | fine | — |
| `rock_spire` | none | — | — | — | fine | — |
| `rolled_sill` | none | — | — | — | fine | — |
| `roof_cresting` | none | — | — | — | fine | — |
| `roof_dormer` | none | — | — | — | fine | — |
| `roof_eaves_overhang_frame` | none | — | — | — | fine | — |
| `roof_hatch` | none | — | — | — | fine | — |
| `rose_window_ring` | none | — | — | — | fine | — |
| `rotating_platform` | none | — | — | — | fine | — |
| `rough_stone_ledge` | none | — | — | — | fine | — |
| `round_bastion` | none | — | — | — | fine | — |
| `round_corner_bay` | none | — | — | — | fine | — |
| `round_deck_dome` | none | — | — | — | fine | — |
| `round_funnel` | none | — | — | — | fine | — |
| `round_tail_light_cluster` | none | — | — | — | fine | — |
| `round_tower` | walk-on | 8 x 8 ring of 48092s; a Plate 8 x 8 Round every floor_every courses (default 3: 9 plates of wall, then a 1-plate floor, so 9 plates clear between floors); merlons a 1 x 2 brick and a tile (4 plates) over the four seams | — | returns the top floor's z, enough for what stands on it | limited: the caller sets floor_every (5 gives 15 plates clear); it has no door | — |
| `round_window` | none | — | — | — | fine | — |
| `round_window_frame` | none | — | — | — | fine | — |
| `rounded_car_end` | none | — | — | — | fine | — |
| `rounded_column` | none | — | — | — | fine | — |
| `rounded_pier_nose` | none | — | — | — | fine | — |
| `rudder_hinge` | none | — | — | — | fine | — |
| `rusticated_stone_pilaster` | none | — | — | — | fine | — |
| `saucer_petal_panels` | none | — | — | — | fine | — |
| `scalloped_awning` | walk-under | reaches 2 studs out from the wall face (plates at y - 1 and y - 2); its underside (the first plate and the round-plate scallops) at z, the caller's; slopes on top to z + 4; the example puts it at z = 9 (a brick course and a 1 x 2 x 2 window) | no height argument: the height is the z the caller picks, set by the window under it | returns z + 1 (the wall's next course); no .underside (z) for a door or a canopy beside it to check against | fixed | says z is the awning's underside, at or above a door's top; returns .underside; the example's awning hangs at 15 (was 9) |
| `ship_hull` | none | — | — | — | fine | — |
| `shipping_container` | none | — | — | — | fine | — |
| `shop_window` | none | — | — | — | fine | — |
| `shuttered_window` | none | — | — | — | fine | — |
| `side_stud_eyes` | none | — | — | — | fine | — |
| `side_stud_lights` | none | — | — | — | fine | — |
| `side_vent_grille` | none | — | — | — | fine | — |
| `sidewalk` | walk-on | tiles at z, top z + 1; depth 3 by default (curb row at y, paving behind); `studs` left bare | — | returns z + 1; the bare studs are the caller's own list, so nothing is missing | fine | — |
| `sideways_chimney_column` | none | — | — | — | fine | — |
| `signboard_beam` | none | — | — | — | fine | — |
| `six_wide_truck_cab` | sit | front to -x; floor top z; cab_seat at (x + 3, y + 2, z): steering wheel holder (30640) x + 3 to x + 5, seat 4079 at (x + 5, y + 2, z), facing -x, seat surface z + 1 (4079 is 8 LDU from its bottom to its seat, 48 to the top of its back); side walls at y and y … | cab size and seat place hard-coded: 8 x 6, roof at z + 9 (dash 3 plates + windscreen 6), seat at x + 5 behind the wheel at x + 3 | returns only the roof's top (z + 10); not the seat's position, which a door or a figure placed in it needs | fixed | a course of plates lifts the roof to 9 plates over the seat (it was a plate inside); returns .underside |
| `skyline_depth_rows` | none | — | — | — | fine | — |
| `slender_column_capital` | none | — | — | — | fine | — |
| `slope_hood_small_window` | none | — | — | — | fine | — |
| `smooth_display_base` | none | — | — | — | fine | — |
| `snot_core` | none | — | — | — | fine | — |
| `snot_panel` | none | — | — | — | fine | — |
| `snot_side_skirt` | none | — | — | — | fine | — |
| `snow_cap` | none | — | — | — | fine | — |
| `snow_drift_edge` | none | — | — | — | fine | — |
| `solar_panel` | none | — | — | — | fine | — |
| `speckled_block_wall` | none | — | — | — | fine | — |
| `sphere_from_caps` | none | — | — | — | fine | — |
| `spiral_dish_stack` | none | — | — | — | fine | — |
| `spliced_chassis` | none | — | — | — | fine | — |
| `stairs` | walk-on | width=2 studs across, rise=3 plates (a brick a step), tread=1 stud; top tread at z + rise x steps | — | returns only the top tread's z, not the landing's (x, y) | fine | — |
| `start_gantry` | walk-under | pillars 2 x 2 at x + 1 to x + 3 and x + 3 + span to x + 5 + span: span=6 studs clear; beam plate's underside at z + 6 x courses = z + 18 (courses=3); the 4 x 4 round bases (1 plate) narrow the floor gap to span - 2 | COURSE = 6 plates a course; courses has no lower bound (2 gives 12 plates clear, 1 gives 6) | doesn't return the pillars' positions or the beam's underside | fine | — |
| `statuette_group` | none | — | — | — | fine | — |
| `stave_cylinder` | none | — | — | — | fine | — |
| `steep_nave_roof` | none | — | — | — | fine | — |
| `steering_rack` | none | — | — | — | fine | — |
| `stepped_block_terrain` | none | — | — | — | fine | — |
| `stepped_curve_hull` | none | — | — | — | fine | — |
| `stepped_plank_roof` | none | — | — | — | fine | — |
| `stepped_tower_setbacks` | none | — | — | — | fine | — |
| `stepping_stones` | walk-on | tiles 2 or 3 studs, top at z + 1; gap=1 stud of grass between; within y to y + wander + 2 | — | — | fine | — |
| `stick_torch` | walk-past | a 1 x 1 stick: on a 2 x 2 jumper (centered), a round plate, height=2 round bricks, 2 flame plates; top at z + 10 | — | — | fine | — |
| `stone_lantern` | walk-past | 2 x 2 plinth; roof plate 4 x 4 reaching a stud past it each way (x - 1 to x + 3); eaves at z + 8 + 3 x pillar (11 plates at pillar=1), finial top about z + 19 | — | — | fixed | the example's second lantern stands 3 studs clear of the first |
| `street_cart` | none | — | — | — | fine | — |
| `string_course` | none | — | — | — | fine | — |
| `studs_up_snow_roof` | none | — | — | — | fine | — |
| `sunken_pond_bank` | none | — | — | — | fine | — |
| `surf_line` | none | — | — | — | fine | — |
| `table_and_chairs` | sit | chair 1 x 1, seat top 3 plates (bracket 36840 at z + 2); table top 5 plates (round: 2 x 2 round tile, underside z + 4; long: tile on a 2 x 4 plate whose underside is z + 3); set 4 x 4 (round) or 6 x 4 (long) | seat (3 plates) and top (5 plates) heights hard-coded in _chair and _set; chair spots in SEATS | returns the set's Handle only, not the chairs' (x, y) and facing | fixed | each chair keeps a stud at each side and 9 plates over its seat clear (the table is where the legs go, by design) |
| `tail_lights` | none | — | — | — | fine | — |
| `tapered_deck` | none | — | — | — | fine | — |
| `tapered_hood_wedges` | none | — | — | — | fine | — |
| `tapered_tower_cap` | none | — | — | — | fine | — |
| `technic_brace` | none | — | — | — | fine | — |
| `technic_brick_ring_chassis` | none | — | — | — | fine | — |
| `technic_pin_hinge` | none | — | — | — | fine | — |
| `technic_spine` | none | — | — | — | fine | — |
| `teeth_row` | none | — | — | — | fine | — |
| `thick_wall` | none | — | — | — | fine | — |
| `tile_brick_texture` | none | — | — | — | fine | — |
| `tile_stripe_shingles` | none | — | — | — | fine | — |
| `tiled_floor` | walk-on | 2 plates thick, top at z + 2 | — | — | fine | — |
| `tiled_pond_water` | none | — | — | — | fine | — |
| `tiled_roof_panel` | none | — | — | — | fine | — |
| `tilting_hood` | none | — | — | — | fine | — |
| `timber_frame_panels` | none | — | — | — | fine | — |
| `torch_sconce` | walk-past | clip brick in the course at z; torch a stud out of the wall (y - 1), reaching about 2 bricks above the course | — | doesn't return the torches' (x, y) | fine | — |
| `tower_crown` | none | — | — | — | fine | — |
| `tower_setback_plinth` | none | — | — | — | fine | — |
| `tower_side_fins` | none | — | — | — | fine | — |
| `track_curve` | none | — | — | — | fine | — |
| `track_straight_run` | none | — | — | — | fine | — |
| `track_switch` | none | — | — | — | fine | — |
| `trailer_hitch` | none | — | — | — | fine | — |
| `train_carriage` | sit | inside: floor at z to the roof plates at z + 10, 10 plates; 4 studs clear between the 1-wide sides (y + 1 to y + 5) | body height hard-coded: a brick course, 1 x 2 x 2 windows (60592), top plates at z + 9, roof plates at z + 10 | returns only the roof's top, not the inside's floor or the roof's underside (z + 10) | fixed | drops the docstring's doors it never built; says the inside fits a seated figure but not a standing one; returns .underside |
| `tree` | none | — | — | — | fine | — |
| `triangle_tile_floor` | none | — | — | — | fine | — |
| `truck_spine_chassis` | none | — | — | — | fine | — |
| `turned_post_porch` | walk-under | tiers 4 (a round brick and a plate each): the fascia's underside at 16 plates, under an 18-plate door top; a rail the whole front | a way in through the rail | the posts' positions and the underside | fixed | tiers 5 by default (20 plates; 4 gave 16), 4 to 8; gap for the way in, kept clear to the house wall; returns .posts and .underside |
| `turret` | none | — | — | — | fine | — |
| `twin_arched_window` | none | — | — | — | fine | — |
| `two_story_color_split` | none | — | — | — | fine | — |
| `two_tone_masonry` | none | — | — | — | fine | — |
| `upright_piano` | sit | played from a seat in front: keys (2431px0) top at z + 5 on the front row; knee gap between the 1 x 1 legs 2 studs wide (x + 1 to x + 3) and 3 plates high (shelf 3020 at z + 3); 4 x 2, lid top z + 11 | keys at 5 plates hard-coded (shelf at z + 3, tile at z + 4), matching table_and_chairs' 3-plate chair | doesn't return where the player's seat goes (in front of the keys' middle) | limited: played from a separate seat; whether a figure's knees fit under its keys is the chair's question | — |
| `vent_fins` | none | — | — | — | fine | — |
| `vintage_chassis` | none | — | — | — | fine | — |
| `vintage_radiator` | none | — | — | — | fine | — |
| `wainscot_split_wall` | none | — | — | — | fine | — |
| `wall_banner` | none | — | — | — | fine | — |
| `wall_lamp_bracket` | walk-past | arm (1 x 2 plate) a stud out at z; lantern hangs from z - 7 (cone) to z, a stud in front of the wall; at studs 0 and length - 1 by default | lantern drop hard-coded (round plate, round brick, round plate, cone 4589 at z - 7); the docstring says 6 plates | doesn't return the lamps' (x, y) or their bottom z | fixed | the example builds its door with door_frame |
| `wall_picture_and_clock` | none | — | — | — | fine | — |
| `wall_shelf` | none | — | — | — | fine | — |
| `wall_top_cornice` | none | — | — | — | fine | — |
| `wall_top_tie` | none | — | — | — | fine | — |
| `wall_vent` | none | — | — | — | fine | — |
| `water_tile_field` | none | — | — | — | fine | — |
| `waterfall` | none | — | — | — | fine | — |
| `wedge_wing` | none | — | — | — | fine | — |
| `white_ashlar_wall` | none | — | — | — | fine | — |
| `window_band` | sit | a passenger car's side 11 plates tall: brick course z, stripe plate z + 3, 1 x 2 x 2 windows z + 4 to z + 10, top plates z + 10 to z + 11; door bays 2 studs wide | window height (60592c01, 2 bricks) hard-coded; a door bay is two stacked dark 1 x 2 bricks (z + 4 to z + 10), a solid panel over the wall course, not an opening | doesn't return the door bays' x or the top as the roof's underside | fixed | says its door bays are panels, not openings; returns .doors |
| `window_box` | none | — | — | — | fine | — |
| `window_frame` | none | — | — | — | fine | — |
| `window_hood` | none | — | — | — | fine | — |
| `window_sill` | none | — | — | — | fine | — |
| `windscreen` | sit | windscreen 3823 2 x 4 x 2 at z; roof plate 4 x 4 at z + 6 (its underside), tiles on top to z + 8 | roof at z + 6 set by the part | returns the roof's top only, not the underside (z + 6) a seat fits under | fixed | returns .underside; the example is a cab a figure fits in |
| `wing_plate_porch_roof` | walk-under | the wing at the caller's z, refused only under 3 plates over the ground; the example at 15 plates; its corbel needs a gap in the wall course | leaving out the corbel on a whole wall | the posts' positions and the underside | fixed | needs 5 bricks of headroom (was 3 plates); corbel=False for a whole wall; says where the door goes; tall posts split into steps (a float z broke them); returns .posts and .underside |
| `wood_stove` | none | — | — | — | fine | — |
| `wooden_hoarding` | walk-on | gallery floor top at z + 4 (beams z to z + 3, floor plate on them); roof plates at z + 10, so 6 plates clear; inside 1 stud deep (y - 1 to y) between the log front (y - 2) and the parapet (y) | beams are 1 x 3 bricks (3622) reaching 2 studs out; front and parapet are 2 courses and a plate (zr = zf + 7) | returns only the roof plates' top, not the gallery floor or the roof's underside | fixed | rows (5 by default, 5 to 8): 15 plates of headroom on the gallery (was 6); returns .floor and .underside |
| `woodpile` | none | — | — | — | fine | — |
| `worm_drive` | none | — | — | — | fine | — |
