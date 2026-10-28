# SPDX-License-Identifier: AGPL-3.0-or-later
"""Rung 2's reference model (plan section 02): a train station with a loop of track running past it."""

from typing import Annotated, Literal

from emmet_dsl import Brick, Color, Model, Plate, Range, Step, model

STRAIGHT = 16  # one straight track, in studs
BASEPLATE = 32  # a 32 x 32 baseplate
GROUND = 0.5  # the top of a baseplate, in plates
TRACK_HALF = 4  # track is 8 studs wide: 4 each side of its center line
HALF_TURN = 8  # curves in a half turn
PLATFORM_DEPTH = 6
BRICK = 3  # plates
DECK_STEP = 16  # studs of platform deck a step: eight 2-wide plates
CANOPY_DEPTH = 4  # the canopy covers the platform behind its edge
BENCH = 4  # a bench's length
BAY = 8  # canopy columns are this far apart; a bench sits between two
BOOTH = (6, 4)  # the ticket booth's width and depth
BOOTH_ROWS = 3  # bricks tall
WINDOW = (60592, 60601)  # Window 1 x 2 x 2 and its glass


def booth(m, x, y, color, roof):
    """The ticket booth behind the platform: a floor, brick walls with a window facing the platform, and a gable."""
    width, depth = BOOTH
    with m.step("Ticket booth: floor"):
        m.add(
            Plate(4, 6), "Dark Bluish Gray", at=(x, y, GROUND), rot=1, id="booth/floor"
        )
    z = GROUND + 1
    window = [(x + 2, y), (x + 3, y)]
    for row in range(BOOTH_ROWS):
        for sides in ("back", "sides", "front"):
            with m.step(f"Ticket booth, row {row + 1}: {sides}"):
                m.wall(
                    x,
                    y,
                    z + BRICK * row,
                    width,
                    depth,
                    color,
                    sides=sides,
                    openings=window if row else (),
                )
        if row == 1:
            with m.step("Ticket booth: window"):
                frame = m.add(
                    str(WINDOW[0]),
                    "White",
                    at=(x + 2, y, z + BRICK),
                    rot=3,
                    id="booth/window",
                )
                m.add(
                    str(WINDOW[1]),
                    "Trans-Clear",
                    at=(x + 2.35, y, frame.min[2] + 0.875),
                    rot=3,
                    id="booth/glass",
                )
    top = z + BRICK * BOOTH_ROWS
    with m.step("Ticket booth: ceiling"):
        m.add(Plate(4, 6), color, at=(x, y, top), rot=1, id="booth/ceiling")
    m.use(
        "gable_roof",
        x=x,
        y=y,
        z=top + 1,
        length=width,
        depth=depth,
        color=roof,
        title="Ticket booth roof",
    )


@model
def station(
    platform_length: Annotated[int, Range(16, 64), Step(8)] = 32,
    canopy: Literal["flat", "gable"] = "flat",
    loop_size: Annotated[int, Range(0, 3)] = 1,
    switch: bool = True,
    benches: Annotated[int, Range(0, 8)] = 3,
    lamps: bool = True,
    platform_color: Color = "Light Bluish Gray",
    canopy_color: Color = "Dark Green",
    roof_color: Color = "Dark Bluish Gray",
):
    """A train station beside a loop of track, on 32 x 32 baseplates.
    platform_length: how long the platform is, in studs
    canopy: the canopy's roof, flat or gabled
    loop_size: how many more straights each long side of the loop has
    switch: a switch and a siding on the far side of the loop
    benches: how many benches stand under the canopy
    lamps: lamp posts at the ends of the platform
    platform_color: the color of the platform
    canopy_color: the color of the canopy's columns and the booth's walls
    roof_color: the color of the roofs
    """
    m = Model()
    plates = -(
        -platform_length // BASEPLATE
    )  # baseplates, side by side under the station
    side = 2 * plates + loop_size  # straights along each long side of the loop
    track_y = (
        TRACK_HALF  # the near side's center line; the loop turns left, away from y = 0
    )

    with m.step("Baseplates"):
        m.repeat(
            lambda: m.add("3811", "Green", at=(0, track_y + TRACK_HALF, 0)),
            plates,
            dx=BASEPLATE,
        )

    # The loop: along +x past the station, a half turn left, back along the far side, and a half turn home.
    p = m.use("track_straight_run", at=(0, track_y, 0), heading=0, count=side)
    p = m.use("track_curve", at=p, count=HALF_TURN, turn="left")
    if switch:
        before = (side - 2) // 2  # the switch's straight way is two straights long
        p = m.use("track_straight_run", at=p, count=before)
        ways = m.use(
            "track_switch", at=p, side="right"
        )  # its branch leaves away from the loop
        siding = m.use(
            "track_curve", at=ways["branch"], count=1, turn="left", title="Siding"
        )
        m.use("track_straight_run", at=siding, count=1, title="Siding")
        p = m.use("track_straight_run", at=ways["straight"], count=side - 2 - before)
    else:
        p = m.use("track_straight_run", at=p, count=side)
    m.use("track_curve", at=p, count=HALF_TURN, turn="left")

    # The platform: centered on the baseplates, its front edge at the track's edge.
    x = (BASEPLATE * plates - platform_length) // 2
    y = track_y + TRACK_HALF
    back = y + PLATFORM_DEPTH - 1
    with m.step("Platform: walls"):
        for wall_y in (y, back):
            m.row(Brick, platform_color, at=(x, wall_y, GROUND), length=platform_length)
    deck = GROUND + BRICK
    for start in range(
        x, x + platform_length, DECK_STEP
    ):  # a step a few plates at a time
        with m.step("Platform: deck"):
            count = min(DECK_STEP, x + platform_length - start) // 2
            m.repeat(
                lambda: m.add(
                    Plate(2, PLATFORM_DEPTH), platform_color, at=(start, y, deck)
                ),
                count,
                dx=2,
            )
    top = deck + 1
    m.use(
        "platform_edge", x=x, y=y, z=top, length=platform_length, color=platform_color
    )

    # Benches between the canopy's columns, facing the track, then the canopy over them.
    under = y + PLATFORM_DEPTH - CANOPY_DEPTH
    bays = min(benches, platform_length // BAY)
    if bays:
        with m.step("Benches"):
            m.repeat(
                lambda: m.use("bench", at=(x + (BAY - BENCH) // 2, under + 1, top)),
                bays,
                dx=BAY,
            )
    m.use(
        "canopy",
        x=x,
        y=under,
        z=top,
        length=platform_length,
        depth=CANOPY_DEPTH,
        spacing=BAY,
        style=canopy,
        columns=canopy_color,
        roof=roof_color,
    )

    # Behind the platform: lamp posts at its ends and the ticket booth in the middle.
    if lamps:
        with m.step("Lamp posts"):
            for lamp_x in (x, x + platform_length - 1):
                m.use("lamp_post", at=(lamp_x, back + 2, GROUND))
    booth(m, x + (platform_length - BOOTH[0]) // 2, back + 2, canopy_color, roof_color)
    return m
