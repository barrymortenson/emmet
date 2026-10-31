# Small Train Station

## What it is

A small train station beside an oval loop of plastic train track, on a grid of green 32 x 32 baseplates.
A gray platform with a yellow safety line runs along the inside of the front straight. A ticket booth stands at
its left end, and a canopy covers the rest of the platform, with two benches under it. Steps with a lamp on each
side lead up from the back. A switch on the back straight leads to a short siding inside the loop, which ends at a
buffer stop. Three trees stand inside the loop.

## Knobs

- platform_length: 20 to 48 studs. The loop stretches to fit it.
- canopy_style: gable or flat.
- loop_size: straights per long side, 2 to 6 (16 studs each).
- station_colors: classic red, heritage green, or modern blue (booth walls, frames, door, columns, roofs, benches).
- siding: the switch, siding, and buffer stop, on or off.

## Design notes

- Track sits on the baseplates at z = 0.5. Each half turn is 8 curves, 80 studs across the center line.
- The platform is 1 brick plus a plate of deck, 4 plates above the track's bottom, with its edge at the track's
  inner edge.
- The booth is 6 x 6 studs and 6 bricks tall, so a 1 x 4 x 6 door fits. It has a hip roof.
- The 9V switch (the library has no plastic one) joins the plastic track. Its step_order findings are allowed:
  track lies flat.
- Colors in each scheme are checked against what the frames, door, and bench lattice were made in.

## Don't

- Don't edit anything in build/; emmet run regenerates it.

<!-- emmet:auto -->
<!-- /emmet:auto -->
