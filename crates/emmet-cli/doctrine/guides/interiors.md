# Interiors
Also: interior, inside, rooms, room, furniture, furnished

Rooms a figure can live in, furnished from few parts, in a building that opens to show them.

## Rules

- Every room says what it's for at a glance: a fireplace and an armchair, a kitchen counter and a table, a bed, a bar,
  a shop counter. Put the main piece against the wall that faces the opening.
- Floors are finished, never bare studs, except where something stands: boards or checkered tiles (`plank_floor`,
  `tiled_floor`), a rug (`floor_rug`). Each room gets its own floor, so the plan reads from above.
- Furniture is 3 to 12 parts a piece, sized for a figure (`guide figure-scale`). A room's furniture lines its walls
  and leaves a figure's path from the door.
- The room is as busy as the outside: in a detailed build no 2 x 2 of floor is empty. Clutter makes it lived in:
  cups, plates, a kettle, books, a basket, a cat, firewood.
- Walls carry things too: shelves, pictures, a clock, a lamp (`wall_shelf`, `wall_picture_and_clock`, `bookcase`,
  `dresser_shelf`).
- Build the furniture in while the walls are 1 to 3 bricks high (the doctrine's step rule), and leave the opening
  (the lift-off roof, the open back) big enough to reach in.

## Rooms

- Living room: `hearth_fireplace` (on the chimney wall, so the flue lines up), an armchair or sofa facing it
  (`armchair_and_sofa`), a floor lamp (`floor_lamp_and_candle`), a rug, a picture. 25 to 40 parts.
- Kitchen: a counter 2 deep along one wall with a stove, sink, and cupboard (`kitchen_counter`, `wood_stove`), a table
  with chairs (`table_and_chairs`), a shelf of jars, a checkered floor. 30 to 50 parts.
- Bedroom: `figure_bed` (2 x 6), a nightstand of a 1 x 1 brick with a candle, a rug, a picture. 20 to 30 parts.
- Shop: `display_case_counter`, shelves of goods, a `cross_display_stand`, a register, a sign inside the window.
- Pub or diner: `bar_counter`, stools (a 1 x 1 round brick under a 2 x 2 round tile), `curved_booth`, a stove hood.
- Music room or parlor: `upright_piano`, `armchair_and_sofa`, a clock.
- Two rooms in a small house: a partition wall with an `interior_arch_doorway`; an upper room on a `loft_half_floor`
  reached by `stairs`.
- Sci-fi rooms: dark tiled floors and smooth panel walls, a `control_console`, a `chamfered_door_portal`, lit panels
  (`guide space`).

## How it opens

A lift-off roof (`lift_off_roof_seat`) or floor (`lift_off_floor`) for a building finished on every side; an open
back for a playset; a hinged wall or roof half (`half_roof_hinge`). Say in the reply how it opens.
