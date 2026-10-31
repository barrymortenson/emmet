# SPDX-License-Identifier: AGPL-3.0-or-later
from math import ceil, floor
from typing import Annotated, Literal

from emmet_dsl import Brick, Model, Plate, Range, Step, Tile, model

TRACK_Z = 0.5  # track stands on the baseplates
CURVE_D = 80  # a half turn of 8 curves is 80 studs across the center line
STRAIGHT = 16  # one straight is 16 studs long
MARGIN = 4  # green around the outer track edge

PLATFORM_DEPTH = 8  # studs, from the track's inner edge back
BOOTH = 6  # the ticket booth is 6 x 6 studs
BOOTH_ROWS = 6  # bricks tall, for a 1 x 4 x 6 door

# Station color schemes: booth walls, door and window frames, door, canopy columns, roofs, benches.
SCHEMES = {
    "classic red": dict(
        walls="Red",
        trim="White",
        door="White",
        columns="White",
        roof="Dark Bluish Gray",
        bench="Reddish Brown",
    ),
    "heritage green": dict(
        walls="Tan",
        trim="Dark Green",
        door="Dark Green",
        columns="Dark Green",
        roof="Reddish Brown",
        bench="Reddish Brown",
    ),
    "modern blue": dict(
        walls="White",
        trim="Black",
        door="Blue",
        columns="Light Bluish Gray",
        roof="Blue",
        bench="Dark Bluish Gray",
    ),
}


@model(
    knobs=["platform_length", "canopy_style", "loop_size", "station_colors", "siding"]
)
def build(
    platform_length: Annotated[int, Range(20, 48), Step(4)] = 32,
    canopy_style: Literal["gable", "flat"] = "gable",
    loop_size: Annotated[int, Range(2, 6)] = 3,
    station_colors: Literal[
        "classic red", "heritage green", "modern blue"
    ] = "classic red",
    siding: bool = True,
):
    """A small train station beside a loop of track, on green baseplates.
    platform_length: how long the platform is, in studs (the loop stretches to fit it)
    canopy_style: the canopy's roof over the platform: gabled or flat
    loop_size: how many straights each long side of the loop has (16 studs each)
    station_colors: the color scheme of the canopy, ticket booth, and benches
    siding: a switch on the back straight with a short siding inside the loop
    """
    m = Model()
    c = SCHEMES[station_colors]

    # Each long side holds the platform; the switch takes two straights' length.
    straights = max(loop_size, ceil(platform_length / STRAIGHT))
    run = straights * STRAIGHT

    # Baseplates: 32 x 32, enough to hold the loop with a margin, the loop centered on them.
    nx = ceil((run + CURVE_D + 8 + 2 * MARGIN) / 32)
    ny = ceil((CURVE_D + 8 + 2 * MARGIN) / 32)
    x0 = (nx * 32 - run) // 2  # where the front straight starts
    y0 = (ny * 32 - CURVE_D) // 2  # the front straight's center line

    for j in range(ny):
        with m.step(f"Baseplates: row {j + 1}"):
            for i in range(nx):
                m.add("3811", "Green", at=(32 * i, 32 * j, 0), id=f"base/{i}-{j}")

    # ---- Track: front straight runs +x, a half turn left, the back straight runs -x, a half turn left.
    p = m.use(
        "track_straight_run",
        at=(x0, y0, TRACK_Z),
        heading=0,
        count=straights,
        title="Track, front",
    )
    p = m.use("track_curve", at=p, count=8, turn="left", title="Track, right curve")
    if siding:
        ways = m.use("track_switch", at=p, side="left", title="Track, back")
        p = ways["straight"]
        if straights > 2:
            p = m.use(
                "track_straight_run", at=p, count=straights - 2, title="Track, back"
            )
    else:
        p = m.use("track_straight_run", at=p, count=straights, title="Track, back")
    m.use("track_curve", at=p, count=8, turn="left", title="Track, left curve")

    if siding:
        s = m.use(
            "track_curve", at=ways["branch"], count=1, turn="right", title="Siding"
        )
        siding_end = m.use("track_straight_run", at=s, count=1, title="Siding")
        # Track lies flat and butts end to end: the switch's box only reaches over its neighbors' ends.
        after_switch = ["siding-curve/1"] + (
            ["track-back-straight/1"] if straights > 2 else []
        )
        m.allow(
            "step_order",
            after_switch,
            "track lies flat; the switch's box only overlaps its neighbors' ends",
        )

    # ---- Platform: inside the loop, along the front straight, centered on it.
    L = platform_length
    px = x0 + (run - L) // 2
    py = y0 + 4  # the track's inner edge
    with m.step("Platform: walls"):
        m.wall(
            px, py, TRACK_Z, L, depth=PLATFORM_DEPTH, color="Light Bluish Gray", rows=1
        )
    for i in range(L // 2):
        with m.step(f"Platform: deck, part {i // 8 + 1}"):
            m.row(
                Plate,
                "Dark Bluish Gray",
                at=(px + 2 * i, py, TRACK_Z + 3),
                length=PLATFORM_DEPTH,
                axis="y",
                width=2,
                id=f"platform/deck/{i}",
            )
    deck = TRACK_Z + 4
    m.use(
        "platform_edge",
        x=px,
        y=py,
        z=deck,
        length=L,
        color="Light Bluish Gray",
        line="Yellow",
        title="Platform edge",
    )

    # ---- Ticket booth, at the left end of the platform, behind the yellow line.
    bx, by = px, py + 2
    m.use(
        "window_frame",
        x=bx,
        y=by,
        z=deck,
        length=BOOTH,
        rows=BOOTH_ROWS,
        offset=2,
        sill=1,
        window="1x2x3",
        facing="front",
        color=c["walls"],
        frame_color=c["trim"],
        glass_color="Trans-Clear",
        title="Booth, front wall",
    )
    m.use(
        "window_frame",
        x=bx,
        y=by + BOOTH - 1,
        z=deck,
        length=BOOTH,
        rows=BOOTH_ROWS,
        offset=2,
        sill=2,
        window="1x2x2",
        facing="back",
        color=c["walls"],
        frame_color=c["trim"],
        glass_color="Trans-Clear",
        title="Booth, back wall",
    )
    m.use(
        "window_frame",
        x=bx,
        y=by + 1,
        z=deck,
        length=BOOTH - 2,
        rows=BOOTH_ROWS,
        offset=1,
        sill=2,
        window="1x2x2",
        facing="left",
        color=c["walls"],
        frame_color=c["trim"],
        glass_color="Trans-Clear",
        title="Booth, left wall",
    )
    top = m.use(
        "door_frame",
        x=bx + BOOTH - 1,
        y=by + 1,
        z=deck,
        length=BOOTH - 2,
        rows=BOOTH_ROWS,
        offset=0,
        facing="right",
        color=c["walls"],
        frame_color=c["trim"],
        door_color=c["door"],
        title="Booth, right wall",
    )
    with m.step("Booth: ceiling"):
        m.fill(Plate, c["walls"], at=(bx, by, top), size=(BOOTH, BOOTH))
    m.use(
        "hip_roof",
        x=bx,
        y=by,
        z=top + 1,
        length=BOOTH,
        depth=BOOTH,
        color=c["roof"],
        title="Booth roof",
    )

    # ---- Benches under the canopy, facing the track, then the canopy over them.
    cx, cl = px + BOOTH, L - BOOTH
    with m.step("Benches"):
        for k, f in enumerate((1 / 3, 2 / 3)):
            bxk = cx + round(f * cl) - 2
            m.use(
                "bench", at=(bxk, by + 1, deck), color=c["bench"], id=f"bench/{k + 1}"
            )
    m.use(
        "canopy",
        x=cx,
        y=by,
        z=deck,
        length=cl,
        depth=PLATFORM_DEPTH - 2,
        spacing=8,
        style=canopy_style,
        columns=c["columns"],
        roof=c["roof"],
        title="Canopy",
    )

    # ---- Steps up to the platform from the back, with a lamp on each side.
    sx, sy = cx + cl // 2 - 2, py + PLATFORM_DEPTH
    m.use(
        "stairs",
        x=sx,
        y=sy,
        z=TRACK_Z,
        steps=2,
        width=4,
        toward="-y",
        rise=2,
        color="Light Bluish Gray",
        tread_color="Dark Bluish Gray",
        title="Platform steps",
    )
    with m.step("Lamps by the steps"):
        for k, lx in enumerate((sx - 2, sx + 5)):
            m.use("lamp_post", at=(lx, sy + 1, TRACK_Z), id=f"lamp/{k + 1}")

    # ---- Buffer stop at the end of the siding.
    if siding:
        ex, ey, _, heading = siding_end
        with m.step("Siding: buffer stop"):
            # The siding runs -x and ends at ex; the stop stands just past it, across the track.
            x = floor(ex) - 2
            y = round(ey) - 3
            m.add(Brick(2, 6), "Black", at=(x, y, TRACK_Z), id="buffer/base")
            m.add(Brick(2, 6), "Black", at=(x, y, TRACK_Z + 3), id="buffer/mid")
            m.add(Plate(2, 6), "Red", at=(x, y, TRACK_Z + 6), id="buffer/top")
            m.add(Tile(1, 6), "White", at=(x + 1, y, TRACK_Z + 7), id="buffer/stripe")

    # ---- Trees inside the loop, clear of the station and the siding.
    m.use(
        "pine_tree",
        x=x0 + 4,
        y=y0 + 32,
        z=TRACK_Z,
        tiers=2,
        trunk=2,
        title="Trees: pine",
    )
    m.use(
        "tree", x=x0 + run // 2, y=y0 + 40, z=TRACK_Z, size="large", title="Trees: oak"
    )
    m.use(
        "tree",
        x=x0 + run - 6,
        y=y0 + 28,
        z=TRACK_Z,
        size="medium",
        title="Trees: small tree",
    )

    return m
