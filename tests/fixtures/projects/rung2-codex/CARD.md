# New Model 3

## What it is

A small figure-scale train station beside a closed R40 oval, standing directly on the table (Base: None). Default: approximately 154 x 111 studs, 210 parts, a 32-stud platform, gabled canopy, two benches, and an open-backed ticket booth.

## Design notes

- Standard 53401 straight and 53400 curved track; 16 curves close the oval. Loop size selects 4–6 straight lengths per side.
- Optional left switch and short siding use the library's compatible 9V switch, 75542-f1; the library has no plastic switch. The siding is inside the loop.
- Controls: platform_length (24–48 studs in steps of 8), canopy_style (flat/gable), loop_size (4–6), station_colors (Classic red/Coastal blue/Forest green), switch_and_siding.
- Platform and canopy lengths change together. Station colors change booth walls and roof colors. The booth stays open at the back for access.
- The layout uses standard track geometry; the station is compressed to two benches and one booth.
- Default and both control extremes validate clean, with zero errors or warnings. Render checked in iso.

## Don't

- Don't edit anything in build/; emmet run regenerates it.
- Don't replace standard track with brick-built rails or break the oval when changing loop size.
- Don't write params.json directly.
- Don't add baseplates while Base is None.

<!-- emmet:auto -->
<!-- /emmet:auto -->
