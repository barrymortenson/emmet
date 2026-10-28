# SPDX-License-Identifier: Apache-2.0
import random
from typing import Annotated

from emmet_dsl import Color, Model, Range, Step, model

# The baseplate is half a plate thick, so everything on it starts at z = 0.5.
GROUND = 0.5

BRICKS = {4: "3010", 3: "3622", 2: "3004", 1: "3005"}  # Brick 1 x n
PLATES = {16: "4282", 12: "2445", 10: "3832", 8: "3034", 6: "3795", 4: "3020"}
PLATES |= {
    3: "3021",
    2: "3022",
    1: "3023b",
}  # Plate 2 x n, and a Plate 1 x 2 for an odd width
SLOPES = {4: "3037", 3: "3038", 2: "3039", 1: "3040b"}  # Slope 45 2 x n
RIDGES = {4: "3041", 2: "3043", 1: "3044c"}  # Slope 45 2 x n double


def lengths(n, sizes):
    """Splits a run of n studs into the longest pieces available, longest first."""
    out = []
    for size in sorted(sizes, reverse=True):
        while n >= size:
            out.append(size)
            n -= size
    return out


# A step adds at most this many parts: the 6+ target (plan section 08's step rules).
STEP = 8


def in_steps(m, title, actions):
    """Runs actions, each (how many parts it adds, a function that adds them), in steps of at most STEP parts. A step
    that has to split is titled "title (1 of 2)", and so on; an action's parts (a frame and its glass) stay together."""
    chunks, count = [[]], 0
    for n, add in actions:
        if chunks[-1] and count + n > STEP:
            chunks.append([])
            count = 0
        chunks[-1].append(add)
        count += n
    for i, chunk in enumerate(c for c in chunks if c):
        with m.step(
            title if len(chunks) == 1 else f"{title} ({i + 1} of {len(chunks)})"
        ):
            for add in chunk:
                add()


def brick_row(m, sizes, x0, y, z, color):
    """One row of bricks along x at y, the longest first."""
    x = x0
    for size in sizes:
        m.add(BRICKS[size], color, at=(x, y, z), rot=1)
        x += size


def fill(m, part, color, frame, size, lift, rot, id):
    """Sets glass in a window or a door in its frame: centered in the frame's box, `lift` plates above its bottom.

    size is the filler's (x, y) footprint at rot=0, from `emmet part`; a frame and its filler share a rotation.
    """
    (x0, y0, z0), (x1, y1, _) = frame.min, frame.max
    w, d = size if rot % 2 == 0 else size[::-1]
    m.add(
        part,
        color,
        at=((x0 + x1 - w) / 2, (y0 + y1 - d) / 2, z0 + lift),
        rot=rot,
        id=id,
    )


def runs(cells, axis):
    """Groups cells into unbroken runs along the axis: [(first cell, length), ...]."""
    out = []
    for c in cells:
        if out:
            (lx, ly), n = out[-1]
            if c == ((lx + n, ly) if axis == "x" else (lx, ly + n)):
                out[-1] = ((lx, ly), n + 1)
                continue
        out.append((c, 1))
    return out


@model
def cottage(
    width: Annotated[int, Range(6, 16)] = 10,
    depth: Annotated[int, Range(4, 12), Step(2)] = 8,
    wall_rows: Annotated[int, Range(6, 9)] = 6,
    wall_color: Color = "Tan",
    roof_color: Color = "Dark Red",
    chimney: bool = True,
    roof: bool = True,
    weathering: Annotated[int, Range(0, 30)] = 0,
    seed: Annotated[int, Range(1, 99)] = 1,
):
    """A small cottage on a green 32 x 32 baseplate.
    width: how many studs wide the house is
    depth: how many studs deep the house is
    wall_rows: how many bricks tall the walls are
    wall_color: the color of the walls
    roof_color: the color of the roof
    chimney: add a brick chimney at the back
    roof: put the roof on (off shows the room inside)
    weathering: how much old brick shows through the walls, in percent
    seed: which arrangement of the old bricks (Shuffle picks another)
    """
    m = Model()
    rng = random.Random(seed)
    # The house's front-left and back-right corners, centered on the baseplate.
    x0, y0 = (32 - width) // 2, (32 - depth) // 2
    x1, y1 = x0 + width - 1, y0 + depth - 1

    # The door in the middle of the front wall, a window in each side wall and the back.
    door_x = x0 + (width - 4) // 2
    side_y = y0 + (depth - 2) // 2
    back_x = x0 + (width - 2) // 2
    door = {(x, y0) for x in range(door_x, door_x + 4)}
    windows = {(x0, side_y), (x0, side_y + 1), (x1, side_y), (x1, side_y + 1)}
    windows |= {(back_x, y1), (back_x + 1, y1)}
    # From 10 studs wide there's room for a window on each side of the door, clear of the corners.
    fronts = [door_x - 2, door_x + 4] if door_x - 2 > x0 and door_x + 5 < x1 else []
    windows |= {(fx + i, y0) for fx in fronts for i in range(2)}
    frames = [
        ("left", (x0, side_y), 2),
        ("right", (x1, side_y), 0),
        ("back", (back_x, y1), 1),
    ]
    frames += [(f"front/{i + 1}", (fx, y0), 3) for i, fx in enumerate(fronts)]

    top = GROUND + 3 * wall_rows
    courses = (depth - 4) // 2 + 1
    ridge_top = top + 1 + 3 * (courses + 1) if roof else top + 3
    # The chimney behind the stove: a stack of bricks from the baseplate to above the ridge, which goes up with the
    # walls a brick a row and gets its top after the roof.
    stack = []
    if chimney:
        z = GROUND
        while z < ridge_top + 3:
            stack.append(z)
            z += 3

    def chimney_bricks(lo, hi):
        for i, z in enumerate(stack):
            if lo <= z < hi:
                m.add(
                    "3004",
                    "Reddish Brown",
                    at=(x1 - 2, y1 + 1, z),
                    rot=1,
                    id=f"chimney/{i + 1}",
                )

    # The door in its frame and a window with its glass, each placed together in the wall.
    def door_in_frame(z):
        frame = m.add("60596", "White", at=(door_x, y0, z), rot=3, id="door/frame")
        fill(m, "60623", "Reddish Brown", frame, (0.3, 3.55), 0.875, 3, "door")

    def window(name, at, turn, z):
        frame = m.add("60592", "White", at=(*at, z), rot=turn, id=f"window/{name}")
        fill(
            m, "60601", "Trans-Clear", frame, (0.2, 1.65), 0.875, turn, f"glass/{name}"
        )

    def walls(row):
        """One row of the walls: the back, then the two sides, then the front, a step each (or more, at most STEP
        parts a step), so each step's bricks are in view."""
        z = GROUND + 3 * row
        # Corners alternate between the front and back walls and the side walls, so the joints overlap row to row.
        if row % 2 == 0:
            spans = [("front", (x0, y0), width, "x"), ("back", (x0, y1), width, "x")]
            spans += [
                ("left", (x0, y0 + 1), depth - 2, "y"),
                ("right", (x1, y0 + 1), depth - 2, "y"),
            ]
        else:
            spans = [("left", (x0, y0), depth, "y"), ("right", (x1, y0), depth, "y")]
            spans += [
                ("front", (x0 + 1, y0), width - 2, "x"),
                ("back", (x0 + 1, y1), width - 2, "x"),
            ]
        gaps = (door if row < 6 else set()) | (windows if 2 <= row < 4 else set())
        bricks = {}
        for wall, (sx, sy), n, axis in spans:
            bricks[wall] = []
            cells = [(sx + i, sy) if axis == "x" else (sx, sy + i) for i in range(n)]
            for (x, y), run in runs([c for c in cells if c not in gaps], axis):
                for size in lengths(run, [2, 1]):
                    # A stone base course, then the walls, with some old brick showing through.
                    old = rng.random() * 100 < weathering
                    color = (
                        "Dark Bluish Gray"
                        if row == 0
                        else "Reddish Brown"
                        if old
                        else wall_color
                    )
                    bricks[wall].append(
                        (BRICKS[size], color, (x, y, z), 1 if axis == "x" else 0)
                    )
                    x, y = (x + size, y) if axis == "x" else (x, y + size)
        for title, wall in (
            ("back wall", "back"),
            ("left wall", "left"),
            ("right wall", "right"),
            ("front wall", "front"),
        ):
            actions = []
            if wall == "back":
                actions += [
                    (
                        len([s for s in stack if z <= s < z + 3]),
                        lambda: chimney_bricks(z, z + 3),
                    )
                ]
            if row == 0 and wall == "front":
                actions += [(2, lambda: door_in_frame(z))]
            if row == 2:
                for name, at, turn in frames:
                    if name.split("/")[0] == wall:
                        actions += [
                            (
                                2,
                                lambda name=name, at=at, turn=turn: window(
                                    name, at, turn, z
                                ),
                            )
                        ]
            for part, color, at, turn in bricks[wall]:
                actions += [
                    (
                        1,
                        lambda part=part, color=color, at=at, turn=turn: m.add(
                            part, color, at=at, rot=turn
                        ),
                    )
                ]
            in_steps(m, f"Walls, row {row + 1}: {title}", actions)

    # Built the way an instruction book builds it: a few parts a step, from the baseplate up.
    with m.step("Baseplate"):
        m.add("3811", "Green", at=(0, 0, 0), id="base")

    walls(0)
    # Outside, while there's room to reach: a stone path from the door, with a flower on each side.
    with m.step("Path and flowers"):
        for k in range(2):
            m.add(
                "3068b",
                "Light Bluish Gray",
                at=(door_x + 1, y0 - 2 - 2 * k, GROUND),
                id=f"path/{k + 1}",
            )
        for side, x in (("left", door_x), ("right", door_x + 3)):
            m.add("3062b", "Green", at=(x, y0 - 2, GROUND), id=f"flower/{side}/stem")
            m.add(
                "6141",
                "Red" if side == "left" else "Yellow",
                at=(x, y0 - 2, GROUND + 3),
                id=f"flower/{side}",
            )

    walls(1)
    # The room, furnished while the walls are two bricks high: a stove in front of the chimney at the back right, a
    # table at the back left with stools when there's room, and a bed along the left wall when the house is deep enough.
    with m.step("Inside: stove"):
        m.add("3003", "Dark Bluish Gray", at=(x1 - 2, y1 - 2, GROUND), id="stove")
        for i in range(2):
            m.add(
                "2412b",
                "Black",
                at=(x1 - 2 + i, y1 - 2, GROUND + 3),
                id=f"stove/top/{i + 1}",
            )
    with m.step("Inside: table and stools"):
        m.add("3062b", "Reddish Brown", at=(x0 + 1, y1 - 1, GROUND), id="table/leg")
        m.add("4032a", "Reddish Brown", at=(x0 + 1, y1 - 2, GROUND + 3), id="table/top")
        if width >= 8:
            m.add("3062b", "Tan", at=(x0 + 3, y1 - 1, GROUND), id="stool/1")
        bed = depth >= 8
        if y1 - 3 > (y0 + 4 if bed else y0):  # in front of the table, clear of the bed
            m.add("3062b", "Tan", at=(x0 + 1, y1 - 3, GROUND), id="stool/2")
    if bed:
        with m.step("Inside: bed"):
            m.add("3001", "White", at=(x0 + 1, y0 + 1, GROUND), id="bed")
            m.add("3068b", "White", at=(x0 + 1, y0 + 1, GROUND + 3), id="bed/pillow")
            m.add(
                "3068b", "Dark Blue", at=(x0 + 1, y0 + 3, GROUND + 3), id="bed/blanket"
            )

    for row in range(2, wall_rows):
        walls(row)

    # Off, the roof and ceiling are left out, so the room inside shows.
    if roof:

        def plate_row(y):
            x = x0
            for size in lengths(width, PLATES):
                # rot=1 lays a 2 x n plate along x; the 1 x 2 is already 1 along x.
                m.add(
                    PLATES[size], wall_color, at=(x, y, top), rot=0 if size == 1 else 1
                )
                x += size

        plates = len(lengths(width, PLATES))
        in_steps(
            m,
            "Ceiling",
            [(plates, lambda y=y: plate_row(y)) for y in range(y0, y0 + depth, 2)],
        )

        # A gable roof: each course steps in one stud from the front and the back, until the two sides meet under a
        # ridge of double slopes. Each side of a course is its slopes, then the bricks behind them up to the middle,
        # at most STEP parts a step.
        slopes, bricks = lengths(width, SLOPES), lengths(width, BRICKS)
        for k in range(courses):
            z = top + 1 + 3 * k
            # The rows of bricks under the next course, split between the two sides.
            inner = list(range(y0 + k + 2, y1 - 1 - k))
            half = (len(inner) + 1) // 2
            for title, y, turn, rows in (
                ("front", y0 + k, 3, inner[:half]),
                ("back", y1 - 1 - k, 1, inner[half:]),
            ):

                def slope_row(y=y, turn=turn, z=z):
                    x = x0
                    for size in slopes:
                        # rot=3 slopes toward -y (the front side), rot=1 toward +y (the back).
                        m.add(SLOPES[size], roof_color, at=(x, y, z), rot=turn)
                        x += size

                actions = [(len(slopes), slope_row)]
                actions += [
                    (
                        len(bricks),
                        lambda ry=ry, z=z: brick_row(m, bricks, x0, ry, z, roof_color),
                    )
                    for ry in rows
                ]
                in_steps(m, f"Roof, course {k + 1}: {title}", actions)
        with m.step("Roof, ridge"):
            x = x0
            for size in lengths(width, RIDGES):
                m.add(
                    RIDGES[size],
                    roof_color,
                    at=(x, y0 + courses, top + 1 + 3 * courses),
                    rot=1,
                )
                x += size

    if stack:
        with m.step("Chimney top"):
            chimney_bricks(top, ridge_top + 3)
    return m
