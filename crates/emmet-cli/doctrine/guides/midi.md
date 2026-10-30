# Midi scale
Also: midi scale, midiscale, architecture, architecture scale, display ship

Between micro and figure scale: display models of ships, spacecraft, and famous buildings, about 1:100 to 1:400,
too small for a figure and too big for one-part details. Judged on its outline and its finish from a step back.

## Rules

- No figure fits, and none needs to: drop doors and furniture; keep windows, decks, and portholes as texture.
- Hide the grid. Surfaces are mostly tiles; a face with stud rows across its whole length reads as a toy. Leave a
  few small bare-stud patches as texture, about 70% smooth to 30% texture (`bare_stud_greeble`).
- Curves by size: under 4 studs across, round bricks and plates stacked on an axle; 6 to 12 studs, four curved-slope
  staves round a core with studs on all sides (`stave_cylinder`); a curved wall or a flat disc, a stack of plate
  outlines each a stud further out, finished with curved slopes (`stepped_curve_hull`).
- Big parts are spliced frames: keels, spines, and towers are two-layer plate beams with their joints offset, joined
  with Technic pins (`spliced_chassis`, `technic_spine`).
- Palette: one main color, a second for panel lines in 10 to 20% of the seams, one accent for engines, lights, and
  markings. Black or dark gray only for the stand and recesses.
- Windows and portholes are a band, not parts: a row of 1 x 1 round tiles, grille tiles, or trans tiles on side studs
  (`porthole_band`, `window_band`, `glow_trim_edge`).
- A display model comes with its stand and a plaque: a low black stand about a third as wide as the model, the
  model on it at 10 to 20 degrees, lifting off; a nameplate tile on a smooth base (`smooth_display_base`).

## Ships and spacecraft

- A steamship's funnels and lifeboats: `round_funnel`, `davit_lifeboat`.
- A sailing ship's hull is round-bilged, dark below, a clean band with portholes above, a wood deck: curved slopes on
  the lower courses, never plain steps. Masts are straight round stacks on an axle; rigging is string or bars.
- A starship or rocket: a few large forms (a saucer, a cylinder, a wedge) joined cleanly, greebles only at engines,
  joints, and undersides; glow strips of trans-light blue tiles a plate deep in a darker frame.

## Buildings (architecture scale)

A window is a 1 x 2 grille tile, a 1 x 1 trans tile, or a groove in a brick; a floor is 2 to 3 plates; a person is
not shown. Texture comes from repeated small parts on the faces: grille tiles, 1 x 1 round plates, cheese slopes,
ingots on side studs. Keep the proportions of the real building more closely than figure scale does: the outline is
the whole point. Present it on a dark base with a nameplate.
