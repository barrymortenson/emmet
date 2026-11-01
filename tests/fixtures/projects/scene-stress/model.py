# SPDX-License-Identifier: AGPL-3.0-or-later
from typing import Annotated

from emmet_dsl import Model, Range, model

# Four buildings on a baseplate, each its own colors: wall, trim, roof.
BUILDINGS = [
    ((1, 1), "Tan", "Dark Red", "Reddish Brown"),
    ((17, 1), "White", "Dark Blue", "Dark Bluish Gray"),
    ((1, 17), "Light Bluish Gray", "Dark Green", "Black"),
    ((17, 17), "Bright Light Yellow", "Medium Azure", "Dark Orange"),
]
SIZE = 12
GLASS = ["Trans-Clear", "Trans-Light Blue", "Trans-Black"]
# Furniture and clutter for the rooms: many small parts in many colors, as a real model of this size has.
FURNITURE = [
    "3024",
    "3023",
    "3022",
    "3021",
    "3070b",
    "3069b",
    "3068b",
    "6141",
    "3062b",
    "3005",
    "3004",
    "3003",
    "3040b",
    "3039",
    "4589",
    "2412b",
    "3794b",
    "15068",
    "11477",
    "98138",
    "85861",
    "87087",
    "4733",
    "3622",
    "3063b",
    "3623",
    "3065",
    "4032b",
    "63864",
]
COLORS = [
    "Red",
    "Blue",
    "Yellow",
    "Green",
    "White",
    "Black",
    "Orange",
    "Lime",
    "Magenta",
    "Purple",
    "Medium Nougat",
    "Sand Green",
    "Dark Tan",
    "Trans-Red",
    "Trans-Yellow",
    "Trans-Green",
    "Metallic Silver",
    "Pearl Gold",
    "Chrome Silver",
    "Coral",
    "Dark Turquoise",
    "Lavender",
]


def lamp(m):
    """A street lamp: a post, a pearl collar, and a glowing globe."""
    with m.step("Post"):
        m.add("3062b", "Black", at=(0, 0, 0), id="foot")
        m.stack("3062b", "Pearl Dark Gray", at=(0, 0, 3), count=3, id="post")
    with m.step("Globe"):
        m.add("6141", "Chrome Gold", at=(0, 0, 12), id="collar")
        m.add("3062b", "Glow In Dark Opaque", at=(0, 0, 13), id="globe")


@model
def block(
    floors: Annotated[int, Range(1, 8)] = 6,
    buildings: Annotated[int, Range(1, 4)] = 4,
):
    """A town block of four buildings: the viewer's 2,000-part stress model, with transparent windows, chrome and
    pearl details, rubber, and street lamps built apart.
    floors: how many floors each building has
    buildings: how many of the four buildings stand (the bench's smaller scenes)
    """
    m = Model()
    with m.step("Baseplate"):
        m.add("3811", "Dark Bluish Gray", at=(0, 0, 0), id="base")
    for b, ((x, y), wall, trim, roof) in enumerate(BUILDINGS[:buildings]):
        name = f"Building {b + 1}"
        # Each floor's walls stand on the floor below's slab (the ground floor's on the baseplate, which is half a
        # plate thick), with the room's furniture on its floor.
        base = 0.5
        for f in range(floors):
            # Windows on every face: cells left open in the middle courses, filled with glass.
            open_cells = [(x + c, y) for c in (2, 3, 8, 9)] + [
                (x + c, y + SIZE - 1) for c in (2, 3, 8, 9)
            ]
            open_cells += [(x, y + c) for c in (2, 3, 8, 9)] + [
                (x + SIZE - 1, y + c) for c in (2, 3, 8, 9)
            ]
            with m.step(f"{name}: floor {f + 1}, floor"):
                if f == 0:
                    for k in range(5):
                        m.add(
                            "3832",
                            trim,
                            at=(x + 1 + 2 * k, y + 1, base),
                            id=f"b{b}/f{f}/floor/{k}",
                        )
                else:
                    m.grid(
                        "3958", trim, at=(x, y, base), nx=2, ny=2, id=f"b{b}/f{f}/floor"
                    )
                    base += 1
            z = base
            with m.step(f"{name}: floor {f + 1}, walls"):
                m.wall(x, y, z, SIZE, depth=SIZE, color=wall, rows=1)
                m.wall(
                    x,
                    y,
                    z + 3,
                    SIZE,
                    depth=SIZE,
                    color=wall,
                    rows=2,
                    openings=open_cells,
                )
                m.wall(x, y, z + 9, SIZE, depth=SIZE, color=trim, rows=1)
                # The glass goes in with the walls: the courses over a window stand on it.
                glass = GLASS[(b + f) % len(GLASS)]
                for i, (cx, cy) in enumerate(open_cells):
                    for row in range(2):
                        m.add(
                            "3005",
                            glass,
                            at=(cx, cy, z + 3 + 3 * row),
                            id=f"b{b}/f{f}/glass/{i}/{row}",
                        )
            with m.step(f"{name}: floor {f + 1}, furniture"):
                floor = z + 1 if f == 0 else z
                for i in range(9):
                    n = b * 97 + f * 31 + i * 7
                    cx, cy = x + 2 + 3 * (i % 3), y + 2 + 3 * (i // 3)
                    m.add(
                        FURNITURE[n % len(FURNITURE)],
                        COLORS[n % len(COLORS)],
                        at=(cx, cy, floor),
                        id=f"b{b}/f{f}/item/{i}",
                    )
            base = z + 12
        top = base
        with m.step(f"{name}: roof"):
            m.grid("3958", trim, at=(x, y, top), nx=2, ny=2, id=f"b{b}/roof/cap")
            for k in range(3):
                m.add(
                    "3037",
                    roof,
                    at=(x + 4 * k, y, top + 1),
                    rot=1,
                    id=f"b{b}/roof/s/{k}",
                )
                m.add(
                    "3037",
                    roof,
                    at=(x + 4 * k, y + SIZE - 2, top + 1),
                    rot=3,
                    id=f"b{b}/roof/n/{k}",
                )
            for k in range(5):
                m.add(
                    "3069b",
                    "Pearl Gold",
                    at=(x + 1 + 2 * k, y + 4, top + 1),
                    id=f"b{b}/roof/trim/{k}",
                )
                m.add(
                    "6141",
                    "Chrome Silver",
                    at=(x + 2 + 2 * k, y + 7, top + 1),
                    id=f"b{b}/roof/vent/{k}",
                )
    lamp_unit = m.submodel("Street lamp", lamp)
    with m.step("Street lamps"):
        for i, (lx, ly) in enumerate(
            [(14, 2), (14, 9), (14, 22), (14, 29), (2, 14), (9, 14), (22, 14), (29, 14)]
        ):
            m.place(lamp_unit, at=(lx, ly, 0.5), id=f"lamp/{i}")
    with m.step("Street"):
        for i in range(16):
            m.add(
                "3069b", "Light Bluish Gray", at=(15, 2 * i, 0.5), id=f"street/ns/{i}"
            )
            # The crossing is the north-south street's.
            if i != 7:
                m.add(
                    "3069b",
                    "White" if i % 2 else "Light Bluish Gray",
                    at=(2 * i, 15, 0.5),
                    rot=1,
                    id=f"street/ew/{i}",
                )
        for i, ty in enumerate((3, 7, 11, 19, 23, 27)):
            m.add("6141", "Rubber Black", at=(16, ty, 0.5), id=f"tires/{i}")
    return m
