# SPDX-License-Identifier: AGPL-3.0-or-later
from typing import Annotated, Literal
from emmet_dsl import Model, model, Range, Step, Brick, Plate, Tile


@model(
    knobs=[
        "platform_length",
        "canopy_style",
        "loop_size",
        "station_colors",
        "switch_and_siding",
    ]
)
def build(
    platform_length: Annotated[int, Range(24, 48), Step(8)] = 32,
    canopy_style: Literal["flat", "gable"] = "gable",
    loop_size: Annotated[int, Range(4, 6)] = 4,
    station_colors: Literal[
        "Classic red", "Coastal blue", "Forest green"
    ] = "Classic red",
    switch_and_siding: bool = True,
):
    """A country station beside a standard R40 train oval.
    platform_length: Platform length in studs.
    canopy_style: Flat or gabled canopy.
    loop_size: Number of straight track lengths on each side.
    station_colors: Coordinated station color scheme.
    switch_and_siding: Include the switch and short siding.
    """
    m = Model()
    length = loop_size * 16
    p = m.use(
        "track_straight_run",
        at=(48, 32, 0, 0),
        count=loop_size,
        title="Station straight",
    )
    p = m.use("track_curve", at=p, count=8, title="East end")
    if switch_and_siding:
        ways = m.use("track_switch", at=p, side="left", title="Siding turnout")
        branch = m.use(
            "track_curve", at=ways["branch"], turn="right", title="Siding return"
        )
        m.use("track_straight_run", at=branch, count=1, title="Short siding")
        p = m.use(
            "track_straight_run",
            at=ways["straight"],
            count=loop_size - 2,
            title="Back straight",
        )
    else:
        p = m.use("track_straight_run", at=p, count=loop_size, title="Back straight")
    m.use("track_curve", at=p, count=8, title="West end")
    sx = 48 + (length - platform_length) // 2
    for x in range(sx, sx + platform_length, 8):
        with m.step(f"Platform: foundations {x}"):
            m.row(Brick, "Light Bluish Gray", at=(x, 16, 0), length=8, width=2)
            m.row(Brick, "Light Bluish Gray", at=(x, 18, 0), length=8, width=2)
            m.row(Brick, "Light Bluish Gray", at=(x, 20, 0), length=8, width=2)
            m.row(Brick, "Light Bluish Gray", at=(x, 22, 0), length=8, width=2)
        with m.step(f"Platform: deck {x}"):
            m.fill(Plate, "Light Bluish Gray", at=(x, 16, 3), size=(8, 8))
    m.use("platform_edge", x=sx, y=22, z=4, length=platform_length)
    wall, roof = {
        "Classic red": ("Red", "Black"),
        "Coastal blue": ("Blue", "Light Bluish Gray"),
        "Forest green": ("Green", "Black"),
    }[station_colors]
    with m.step("Platform furniture"):
        m.use("bench", at=(sx + 4, 18, 4), color="Reddish Brown", id="bench/1")
        m.use(
            "bench",
            at=(sx + platform_length - 8, 18, 4),
            color="Reddish Brown",
            id="bench/2",
        )
    m.use(
        "canopy",
        x=sx + 2,
        y=16,
        z=4,
        length=platform_length - 4,
        depth=6,
        spacing=8,
        style=canopy_style,
        columns="White",
        roof=roof,
    )
    bx = sx + platform_length // 2 - 4
    with m.step("Ticket booth: floor"):
        m.fill(Plate, "Light Bluish Gray", at=(bx, 6, 0), size=(8, 6))
    for row in range(2):
        m.wall(
            bx,
            6,
            1 + row * 3,
            length=8,
            depth=6,
            color=wall,
            sides=["front", "left", "right"],
            rows=1,
        )
    for row in range(3):
        with m.step(f"Ticket booth: upper sides {row + 1}"):
            m.row(Brick, wall, at=(bx, 7, 7 + row * 3), length=5, axis="y")
            m.row(Brick, wall, at=(bx + 7, 7, 7 + row * 3), length=5, axis="y")
            m.add(Brick(1, 2), wall, at=(bx, 6, 7 + row * 3), rot=1)
            m.add(Brick(1, 2), wall, at=(bx + 6, 6, 7 + row * 3), rot=1)
    with m.step("Ticket booth: service window"):
        m.add("60594", "White", at=(bx + 2, 6, 7), rot=1, id="ticket-window")
    with m.step("Ticket booth: roof beams"):
        m.row(Plate, "White", at=(bx, 6, 16), length=8)
        m.row(Plate, "White", at=(bx, 11, 16), length=8)
        m.row(Plate, "White", at=(bx, 7, 16), length=4, axis="y")
        m.row(Plate, "White", at=(bx + 7, 7, 16), length=4, axis="y")
    with m.step("Ticket booth: roof"):
        m.fill(Plate, roof, at=(bx, 6, 17), size=(8, 6))
    with m.step("Ticket booth: roof finish"):
        m.fill(Tile, roof, at=(bx, 6, 18), size=(8, 6))
    return m
