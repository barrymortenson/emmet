# Figure scale
Also: figure, minifig, minifigure, minifig scale, minifigure scale, system

The scale of most official sets: everything a 4-brick-tall figure touches is sized for it, and the whole is
shortened to stay buildable (the doctrine's "Sized for the figure, not to the ratio").

## Sizes a figure needs

- A figure is 4 bricks tall with its hat, 2 studs wide at the shoulders, 1 deep. It stands on one stud of its feet
  (2 studs under it), sits on a 1 x 2 or 2 x 2 seat 1 brick high.
- Door: a 1 x 4 x 6 door in its frame (6 bricks), sill 0 to 1 plate above the floor. Its top lines up with the tops of
  the windows.
- Windows: 1 x 2 x 2 or 1 x 2 x 3 frames, sill 1 to 2 bricks above the floor; 1 x 4 x 3 for a shop or a living room.
- Story: 5 to 6 bricks of wall (6 to 7 on a ground floor with a shop), plus a 2-plate floor or ceiling.
- Rooms: at least 6 studs deep where it closes all around (a lift-off roof); 4 to 6 deep when it's open at the back.
  Corridors and doorways inside 4 studs wide, stairs rising 2 to 3 plates a stud.
- Furniture: a seat 1 brick high, a table top 2 to 3 plates above that, a bed 2 x 6, a counter 1 brick and a plate
  high and 2 deep, a fireplace 4 wide and 2 deep.
- Vehicles: a seated figure needs a 2 x 2 footprint and 3 bricks of headroom from its seat plate (`guide vehicles`).

## Clearances

What an experienced builder leaves without being told (`docs/spec/building-rules.md`). The validate tool finds a part
in a space a technique or `m.keep_clear` declared; the rest is yours to see in the render.

- Headroom: anything a figure walks under (a canopy, an awning, a porch roof, an arch) starts at or above the top of
  the door beside it, 6 bricks for a 1 x 4 x 6 door, and 5 bricks at least anywhere. `canopy`, `porch_posts`, and
  `turned_post_porch` clear a door at their defaults; give `lean_to_awning` `clears=` the door's top.
- A door's way through: its width, 3 studs out on both sides, up to its top, stays clear. `door_frame` keeps it and
  returns `.door_top`; a door you place by hand gets `m.keep_clear` for it.
- Seats: a stud free at each end, 2 studs in front, 3 bricks over the seat. Center a bench between two posts, never
  against one: `canopy` returns `.columns`, the porches `.posts`.
- Posts and columns stand 3 studs or more clear apart, never in line with a door.
- A lower roof meets a taller wall under its eaves or on a string course, never across a window or a door. Door tops,
  window tops, and eaves line up along a facade; every roof edge overhangs a stud and meets the wall under it.

## Compress the rest

- Keep the outline and the figure-facing details; drop windows, floors, bays, and length until it fits the Pieces
  setting. A real house is three or four times longer than its model; a ship or a starship ten times.
- Raise the building on a plinth or a step (1 brick, or 2 plates) so it sits in its setting, not on the table.
- The building covers at most half its base: the rest is garden, street, or landscape (`guide landscaping`).

## How it opens

A closed building has to open, or the inside is wasted. Pick one, in this order:

1. A roof that lifts off as one piece on a seat (`lift_off_roof_seat`), with any upper floor lifting off too
   (`lift_off_floor`). Best for a model that has to look finished from every side.
2. Open at the back (a dollhouse): rooms face the open side and furniture lines the other walls. Best for play.
3. A hinged half: a roof side, a wall, or a hull section on hinges (`hinge_roof`, `half_roof_hinge`), or the building split in two halves that swing apart.

## Detail by the Detail setting

- Low: bricks, plates, and slopes; real doors and windows; one feature per face.
- Medium: plus tiles on every flat top that isn't for standing on, sills and hoods around windows, a texture on one
  wall course, furniture in each room.
- High: plus sideways-built textures and trim (studs on the side), two or three wall textures, clutter in rooms,
  landscaping on every free stud. About one small detail every 2 to 3 studs of face.

## Steps

Base, then the floor and a row or two of walls, then the furniture while the walls are low, then the rest of the
walls with their windows, then the roof as a sub-assembly. The doctrine's step rules hold at every scale.
