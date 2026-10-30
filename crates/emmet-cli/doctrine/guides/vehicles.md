# Vehicles
Also: vehicle, car, cars, truck, trucks, van, bus, train, trains, kart, racing car, sports car

Cars, trucks, and trains at every scale. Pick the width class first: it decides the wheels, the lights, the glass,
and where the figure sits.

## Width classes (figure scale)

| | 4 wide | 6 wide | 8 wide | Vintage |
|---|---|---|---|---|
| What | small city car, cart, forklift | truck, van, bus, train | racing-scale replica | 1920s-30s car |
| Length | 8 to 12 | 14 to 22 (train car 28 to 32) | 18 to 22 | 14 to 18 |
| Wheels | small, on wheel-holder plates | 14 to 18 mm on wheel-holder plates | low-profile car wheels on axles in 2 x 2 axle bricks | spoked, narrow tires |
| Arches | none or a 2 x 4 mudguard plate | mudguard plates | shaped by the body: inverted curves | one big fender a wheel |
| Lights | 1 x 1 round on a side stud or a 1 x 1 trans tile | 1 x 1 on a side-stud brick, a grille between | quarter tiles or curved plates, swept | round pods on brackets |
| Glass | 2 x 4 x 2 windscreen or roll cage | 2 x 6 x 2 windscreen | sloped 6-wide screen on a hinge | short upright screen |
| Figure | sits on the plate, open top | 2 x 2 seat, roof lifts off | one figure on the left seat, roof or screen lifts | a bench, open top |

Use `car_wheels` for the wheels and arches at each width, and `four_wide_car` for a whole small car.

## Rules

- Faces with lights and a grille are built on studs that point forward (side-stud bricks, brackets, or a front built
  flat and stood up). Never stack lights as studs-up 1 x 1 bricks (`headlights`, `side_stud_lights`,
  `quarter_tile_headlights`, `front_grille`, `front_bumper`, `tail_lights`).
- The figure decides the cab: a 2 x 2 footprint, 3 bricks of headroom from the seat plate, a steering wheel a stud in
  front, and a roof or glass that lifts off (`cab_seat`, `windscreen`, `six_wide_truck_cab`, `cab_over_sleeper`,
  `glass_box_cab`).
- Length is compressed: trucks and trains stay long (that's their character), cars stay short.
- One body color, black for the chassis, arches, and bumpers, gray for machinery, one accent for stripes. Trans-clear
  at the front, trans-red at the back, trans-orange on the roof.
- City trucks show some studs (the bed, the roof); racing-scale cars show almost none (tiles and curved slopes:
  `curved_wedge_nose`, `tapered_hood_wedges`, `snot_side_skirt`, `rounded_car_end`, `bracket_side_panel`).
- Use plain, grille, and mosaic tiles where the real set uses stickers; never fake a sticker with many tiny parts.
- Chassis: `truck_spine_chassis`, `spliced_chassis` for long ones, `racing_scale_base` for 8 wide, `vintage_chassis`
  with a `vintage_radiator` at its front. A cab whose windows and roof lift off as one: `lift_off_window_cab`. A cart
  or a stall on wheels: `street_cart`.
- Trains: `train_carriage` on a train base, `window_band`, `flat_wagon`, `shipping_container`; body 6 wide, 8 to 9
  bricks to the roof from the rail, roof in lift-off sections.

## Other scales

- Micro and micropolis: a car is 1 x 2 or 2 x 4 studs: a plate with a tile or a slope, no figure, no working wheels
  below 2 x 4 (`guide micro`).
- Large display cars (1:12 to 1:16, 18 to 20 studs wide): `guide large`.
- Technic cars (1:8 to 1:14): `guide car-1-8`.
