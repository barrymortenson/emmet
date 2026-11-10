# Large scale
Also: large scale, display, display scale, big, 1:12, 1:16

Bigger than figure scale: display models of cars, buildings, and objects, built for a shelf. No figure has to fit;
the model is judged on its shape and finish from a step back. Where one has figure doors or seats (a display house
with figures in it), they keep figure scale's clearances (`guide figure-scale`): a door's way through, a seat's room,
and headroom at or above the door's top.

## Rules

- Hide the grid. Almost no visible studs: tiles on flat tops, curved slopes on every edge, faces built sideways in
  every direction, at every Detail setting: at this scale sideways building is the shape, not a texture. A face with
  rows of studs across it reads as a toy. Leave a few small bare-stud patches only where
  the real thing has texture (`bare_stud_greeble`).
- A skeleton inside. Big models hang their shell on a frame of Technic bricks or beams, and the mechanisms run through
  the frame, not the shell (`technic_brick_ring_chassis`, `beam_spine_floor`, `technic_spine`, `snot_core`).
- Big panels built apart and clipped on: a car's sides, doors, hood, and roof, a building's facade. That's also how
  they open (`hang_on_side_panel`, `front_hinged_hood`, `removable_soft_top` for a convertible's roof).
- Curves are runs of curved slopes with the inverted ones under them for the other half of the roll; wedge plates
  taper the outline from above; an inverted arch makes a wheel well (`rolled_sill`, `fender_crown`,
  `inverted_arch_wheel_well`, `tapered_deck`, `canopy_hump`).
- One main color, a lighter interior color, pearl silver or chrome for trim, trans-clear lights. Hidden structure may be
  any color.
- Interiors are complete where they show: seats, a dashboard, a steering wheel, a gear lever, pedals
  (`pleated_seat`, `dashboard_and_wheel`).
- Expect 1,500 to 4,000 parts; ask before going past 2,000 (the doctrine).

## Display cars (1:12 to 1:16)

About 45 to 52 studs long, 18 to 20 wide, 10 to 12 bricks to the roof. Wheels about 4 studs across, the axle line 2
studs up, the floor a brick above it (`large_display_wheel`). Headlights 2 studs across in a silver ring, round red
tail lights, a grille of grille tiles across 6 to 10 studs in a chrome frame (`ring_light_grille`,
`round_tail_light_cluster`, `side_vent_grille`, `flared_exhaust`). Steering through a rack (`steering_rack`).

## Display bikes (1:6 to 1:10)

A motorbike, a dirt bike, or a moto-style e-bike, about 30 to 45 studs long on motorcycle wheels or fat off-road ones
(about 12 studs across), standing on its wheels and its kickstand, with no base: the parts go into the bike. Its
skeleton is Technic beams on edge, never stacked bricks, and every joint is a pin or an axle in a hole: part_info gives
a part's holes by name, `m.beam` lays a beam from one hole to another, and `m.insert` seats a pin or an axle in a named
hole, so no test build is needed to find a hole.

Measure the photo before you write a line, with a wheel's diameter as the ruler (about 12 studs), each point as (run,
rise) in studs from the swingarm's pivot: the steering head (where the top tube and the down tube meet it), the
bottom bracket (the crank), the top tube's back end over the seat tube, and the rear axle; and how far the fork leans.
`motorbike_frame` takes those as head, crank, seat, rear, and rake, fits whole-stud beams to them with the top tube
level, and picks the swingarm's angle so its shock fits; `swingarm_shock` takes the reach (pivot to rear axle) and the
wheel ("fat" or "motorcycle"), and `suspension_fork` sets the front axle level with the rear one. Put the pivot the
swingarm's drop plus the wheel's radius above the table, so the wheels stand on it. A technique's doc is its whole
interface: if it doesn't fit, use its options, or build that part yourself. Build it in this order:

- The frame (`motorbike_frame`): five beams a side, a level top tube, every cross axle held at its ends, and a head
  bearing that can't tip.
- The swingarm and its one shock, which is the rear spring, and the rear wheel; then the chain from the crank to the
  rear sprocket, with the pedals, and the kickstand on the left swingarm arm (`swingarm_shock`, `chain_drive`,
  `kickstand`).
- The fork, which steers and springs: a triple tree on a steerer turning in the frame's head bearing, the fork tubes
  locked in it, sliders on the tubes carrying the front axle, and a shock each side as the spring; then the brakes,
  the front fender, and the bars on the tubes' tops with their grips and levers (`suspension_fork`, `disc_brake`,
  `front_fender`, `handlebars`). Nothing else joins the fork to the frame, so the bars turn it.
- Then the bodywork, hung from the frame, in the subject's colors and shapes: the seat and rack on a spine pinned to
  the top tubes, with the rear fender and tail lights (`bike_seat_rack`), the e-bike's battery under it in the frame's
  triangle (`battery_case`), or the tank; then what makes it this bike: the motor or the hub, side panels and number
  boards, the battery's trim and its lock, lights, cables, the exhaust, footpegs, and mirrors.

Every moving part works as the real one does: the wheels spring against the shocks (never hang on them), the fork
turns, and every axle has a bush, a half bush, a crank arm, or a cross block on it, so nothing slides out. The
techniques give about 230 parts; the rest is what makes it this bike. Count the parts before you call it done: a
display bike is 600 to 2,000 at any Detail setting, in the upper half of the Pieces setting's range when there is
one; a bike at 260 parts is the techniques with little added.

## Other subjects

A large building or object: a stand or base with a nameplate (`smooth_display_base`); see `guide sculpture` for round
and organic forms, and `guide midi` for display ships.
