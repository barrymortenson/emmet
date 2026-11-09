# Large scale
Also: large scale, display, display scale, big, 1:12, 1:16

Bigger than figure scale: display models of cars, buildings, and objects, built for a shelf. No figure has to fit;
the model is judged on its shape and finish from a step back. Where one has figure doors or seats (a display house
with figures in it), they keep figure scale's clearances (`guide figure-scale`): a door's way through, a seat's room,
and headroom at or above the door's top.

## Rules

- Hide the grid. Almost no visible studs: tiles on flat tops, curved slopes on every edge, faces built sideways in
  every direction. A face with rows of studs across it reads as a toy. Leave a few small bare-stud patches only where
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
(about 12 studs across), standing on a stand with a nameplate. Its skeleton is Technic beams on edge, never stacked
bricks, and every joint is a pin or an axle in a hole: part_info gives a part's holes by name, `m.beam` lays a beam
from one hole to another, and `m.insert` seats a pin or an axle in a named hole, so no test build is needed to find a
hole.

Measure the photo before you write a line, with a wheel's diameter as the ruler (about 12 studs): the wheelbase (two
diameters on an e-bike or a dirt bike, more on a road bike or a chopper), the swingarm's pivot from the rear axle,
where the steering head and the frame's lower front corner are from the pivot, how far the fork leans, and how high
the bars are. The techniques take those numbers, so the skeleton is this bike's, not a stock one: `motorbike_frame`'s
head, bottom, and rake; `swingarm_shock`'s reach, drop, and wheel ("motorcycle" or "fat"); `suspension_fork`'s drop
(the swingarm's) and wheel; `handlebars`' rise; and `display_stand_nameplate`'s under (the drop plus the wheel's
radius). A technique's doc is its whole interface: if it doesn't fit, use its options, or build that part yourself.
Build it in this order:

- The stand, then the frame hung on its post (`display_stand_nameplate`, `motorbike_frame`): a triangle of beams a
  side, and a head bearing raked forward.
- The swingarm with its twin shocks and the rear wheel, then the chain from the crank to the rear sprocket, with the
  pedals (`swingarm_shock`, `chain_drive`).
- The fork, which steers: a triple tree on a steerer turning in the frame's head bearing, the fork tubes, their
  shocks, and the front wheel; then its brakes, and the bars on the tubes' tops with their grips and levers
  (`suspension_fork`, `disc_brake`, `handlebars`). Nothing else joins the fork to the frame, so the bars turn it.
- Then the bodywork, on the frame's holes and hung from it, in the subject's colors and shapes: the tank or the
  e-bike's battery in the frame's triangle, the motor, the seat and its subframe or rack, side panels and number
  boards, fenders over both wheels, lights, the exhaust, footpegs, and mirrors. The skeleton is about 200 parts; the
  rest is what makes it this bike. Count the parts before you call it done: a display bike is 600 to 2,000.

## Other subjects

A large building or object: a stand or base with a nameplate (`smooth_display_base`); see `guide sculpture` for round
and organic forms, and `guide midi` for display ships.
