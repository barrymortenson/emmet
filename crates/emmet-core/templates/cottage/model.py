# SPDX-License-Identifier: Apache-2.0
import itertools
import random
from typing import Annotated, Literal

from emmet_dsl import Brick, Color, Model, Plate, Range, Rot, Step, Tile, frame, model
from emmet_dsl.core import (
    in_steps,
)  # steps of at most the Age setting's target: 4 for 4+, 12 for 18+, 8 otherwise

# The baseplate is half a plate thick, so everything on it starts at z = 0.5.
BASEPLATE = 0.5

SLOPES = {4: "3037", 3: "3038", 2: "3039", 1: "3040b"}  # Slope 45 2 x n
COURSES = {8: "3007", 4: "3001", 3: "3002", 2: "3003"}  # Brick 2 x n
TILES = {4: "2431", 3: "63864", 2: "3069b", 1: "3070b"}  # Tile 1 x n
BRICKS = {4: "3010", 3: "3622", 2: "3004", 1: "3005"}  # Brick 1 x n
# The stone courses: Bricks 1 x 4 and 1 x 2 with embossed bricks, and a plain 1 x 1 where a run needs one.
EMBOSSED = {4: "15533", 2: "98283", 1: "3005"}
# Siding: Bricks 1 x 4 and 1 x 2 Log, whose grooves read as boards, and a plain 1 x 1 where a run needs one.
LOGS = {4: "30137", 2: "30136", 1: "3005"}

# The walls' brick lengths: 1 x 4 and shorter come in the most colors.
WALL_BRICKS = [1, 2, 3, 4]


def lengths(n, sizes):
    """Splits a run of n studs into the longest pieces available, longest first."""
    out = []
    for size in sorted(sizes, reverse=True):
        while n >= size:
            out.append(size)
            n -= size
    return out


def staggered(n, sizes, course):
    """Like lengths, but every other course starts with a 2-long piece, so its joints mostly fall between the ones
    before it instead of on them."""
    if course % 2 and n > 2 and 2 in sizes:
        return [2] + lengths(n - 2, sizes)
    return lengths(n, sizes)


def piece(m, part, color, at, rot=0, id=None):
    """One part as an in_steps action."""
    return (1, lambda: m.add(part, color, at=at, rot=rot, id=id))


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


def stone_course(
    m,
    x,
    y,
    z,
    length,
    depth,
    color,
    sides=("back", "left", "right", "front"),
    openings=(),
    pieces=EMBOSSED,
):
    """One course of embossed brickwork (or other `pieces`: {length: part}) round the walls of a length x depth
    rectangle from (x, y), at z. Like m.wall, the front and back walls own the corners on even courses and the sides on
    odd ones, so the corners interlock; each run between openings is the longest pieces, starting with a 1 x 2 on odd
    courses so its joints move over."""
    course = round(z / 3) % 2
    x1, y1 = x + length - 1, y + depth - 1
    lines = {
        "front": [(cx, y) for cx in range(x + course, x1 + 1 - course)],
        "back": [(cx, y1) for cx in range(x + course, x1 + 1 - course)],
        "left": [(x, cy) for cy in range(y + 1 - course, y1 + course)],
        "right": [(x1, cy) for cy in range(y + 1 - course, y1 + course)],
    }
    for side in sides:
        axis = "x" if side in ("back", "front") else "y"
        for (cx, cy), n in runs([c for c in lines[side] if c not in openings], axis):
            for size in staggered(n, pieces, course):
                m.add(pieces[size], color, at=(cx, cy, z), rot=1 if axis == "x" else 0)
                cx, cy = (cx + size, cy) if axis == "x" else (cx, cy + size)


# Each wall faces out one way. TURN turns a part that faces front (-y) to face the wall's way; a side stud's brick
# (87087, 4733's studs) points the way TURN names, and a tile on a side stud lies on its back with TILE_ROT.
TURN = {"-y": 0, "+x": 1, "+y": 2, "-x": 3}
STUD_ROT = {"+x": 0, "+y": 1, "-x": 2, "-y": 3}
TILE_ROT = {
    "-y": Rot(y=-90, z=90),
    "+x": Rot(y=-90, z=180),
    "+y": Rot(y=-90, z=270),
    "-x": Rot(y=-90),
}
OUT = 0.4  # a tile's thickness, in studs


def face_tile(m, color, cell, face, z, n=1, back=False):
    """A 1 x n tile on the side studs of the course at z in cell, standing proud of the wall that faces `face`. It runs
    along the wall from the cell, or back toward lower x or y with `back` (a corner's tile reaches along its wall)."""
    cx, cy = cell
    if face in ("-y", "+y"):
        at = (
            cx - (n - 1 if back else 0),
            cy - OUT if face == "-y" else cy + 1,
            z + 0.5,
        )
    else:
        at = (
            cx - OUT if face == "-x" else cx + 1,
            cy - (n - 1 if back else 0),
            z + 0.5,
        )
    m.add(TILES[n], color, at=at, rot=TILE_ROT[face])


def door_in_frame(m, color):
    """The door sub-assembly: a door frame with its door."""
    with m.step("Frame and door"):
        f = m.add("60596", "White", at=(0, 0, 0), id="frame")
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60623",
            color,
            at=((x0 + x1 - 0.3) / 2, (y0 + y1 - 3.55) / 2, z0 + 0.875),
            id="leaf",
        )


def window_unit(m, shutters, trim, wall, shutter, flowers, lights=False):
    """A window sub-assembly, built along x in a wall facing front (-y) whose line is y 0 to 1, its hood over the
    wall's cells x 0 to 4. The window 1 x 2 x 3 stands on cells 1 and 2 over a window box (window_box), its top under
    the hood (window_hood). With shutters, the unit is all four cells from the band course (z 0) up: a plate 2 x 4 in
    the band's color whose front row carries the box, a column each side of two bricks with studs on one side under a
    louvered shutter, the window, and a plate over it. Without, it starts a plate higher (the sill course is z 0),
    and the wall's own courses fill cells 0 and 3. Lit, the window is a 1 x 2 x 2 with yellow glass under a brick, as
    if the lamps were on behind it."""
    if shutters:
        with m.step("Box plate"):
            m.add("3020", trim, at=(0, -1, 0), rot=1, id="plate")
            m.add("11211", trim, at=(1, -1, -3), rot=3, id="box")
        with m.step("Flowers"):
            m.add("33291", flowers[0], at=(0.9, -1.4, -2.75), rot=Rot(x=90))
            m.add("6141", "Bright Green", at=(2, -1.4, -2.5), rot=Rot(x=90))
            m.add("3069b", trim, at=(1, -1, 1), rot=1, id="sill")
        for side, cx in (("left", 0), ("right", 3)):
            with m.step(f"Shutter, {side}"):
                for k in range(2):
                    m.add("32952", wall, at=(cx, 0, 1 + 5 * k), rot=3)
                    m.add(
                        "2412b",
                        shutter,
                        at=(cx, -OUT, 1 + 5 * k),
                        rot=Rot(x=90),
                        id=f"shutter/{side}/{k + 1}",
                    )
        z = 1
    else:
        z = m.use(
            "window_box",
            x=1,
            y=0,
            z=0,
            length=2,
            color=trim,
            flowers=flowers,
            title="Window box",
        )
    with m.step("Frame and glass"):
        f = m.add(
            "60592" if lights else "60593", "White", at=(1, 0, z), rot=1, id="frame"
        )
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601" if lights else "60602",
            "Trans-Yellow" if lights else "Trans-Clear",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
            id="glass",
        )
        if lights:
            m.add("3004", "White", at=(1, 0, z + 6), rot=1, id="header")
    if shutters:
        with m.step("Over the window"):
            m.add("3023b", "White", at=(1, 0, z + 9), rot=1)
    m.use(
        "window_hood",
        x=1,
        y=0,
        z=10 if not shutters else 11,
        width=2,
        color=trim,
        title="Hood",
    )


def lit_window(m, at, along_x, color):
    """A lit window 1 x 2 x 3 high: a Window 1 x 2 x 2 with yellow glass, and a brick 1 x 2 over it."""
    f = m.add("60592", color, at=at, rot=1 if along_x else 0)
    (x0, y0, z0), (x1, y1, _) = f.min, f.max
    if along_x:
        g = ((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875)
    else:
        g = ((x0 + x1 - 0.2) / 2, (y0 + y1 - 1.65) / 2, z0 + 0.875)
    m.add("60601", "Trans-Yellow", at=g, rot=1 if along_x else 0)
    m.add("3004", color, at=(at[0], at[1], at[2] + 6), rot=1 if along_x else 0)


def lit_bay(m, x, y, z, color, trim, roof, title):
    """bay_window (6 wide, no plinth) with its windows lit: the same floor, posts, cap, and lean-to roof."""
    front, right = y - 3, x + 5
    with m.step(f"{title}: floor and posts"):
        m.add("3032", trim, at=(x, front, z), rot=1)
        for cx in (x, right):
            m.add("14716", color, at=(cx, front, z + 1))
            m.add("14716", color, at=(cx, y, z + 1))
    with m.step(f"{title}: side windows"):
        for cx in (x, right):
            lit_window(m, (cx, front + 1, z + 1), False, color)
    with m.step(f"{title}: front windows"):
        for k in range(2):
            lit_window(m, (x + 1 + 2 * k, front, z + 1), True, color)
    with m.step(f"{title}: cap"):
        m.add("3032", trim, at=(x, front, z + 10), rot=1)
    with m.step(f"{title}: roof"):
        for k in range(0, 6, 2):
            m.add("3039", roof, at=(x + k, front, z + 11), rot=3)
        m.add("3009", roof, at=(x, y - 1, z + 11), rot=1)


def lit_dormer(m, x, y, z, color, roof, title):
    """roof_dormer with its window lit: returns the function that adds its gable."""
    with m.step(f"{title}: window"):
        for r in range(2):
            m.add("3005", color, at=(x, y, z + 3 * r))
            m.add("3005", color, at=(x + 3, y, z + 3 * r))
        f = m.add("60592", color, at=(x + 1, y, z), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601",
            "Trans-Yellow",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
        )
        m.add("3010", roof, at=(x, y + 1, z), rot=1)

    def gable():
        with m.step(f"{title}: gable"):
            m.add("3039", roof, at=(x, y, z + 6), rot=2)
            m.add("3039", roof, at=(x + 2, y, z + 6), rot=0)

    return gable


def place_on_wall(m, unit, face, cell, n, z, id):
    """Places a unit built along x facing front (as window_unit is) on the wall facing `face`, over the n cells of that
    wall from `cell` (its lowest x or y), with its own z = 0 at z."""
    cx, cy = cell
    k = TURN[face]
    anchor = {
        "-y": (cx, cy),
        "+y": (cx + n, cy + 1),
        "+x": (cx + 1, cy),
        "-x": (cx, cy + n),
    }[face]

    def turn(x, y):
        for _ in range(k):
            x, y = -y, x
        return x, y

    (bx0, by0, bz0), (bx1, by1, _) = unit.box
    corners = [turn(x, y) for x in (bx0, bx1) for y in (by0, by1)]
    at = (
        anchor[0] + min(c[0] for c in corners),
        anchor[1] + min(c[1] for c in corners),
        z + bz0,
    )
    return m.place(unit, at=at, rot=k, id=id)


# Plant parts' bodies in LDraw units around the stud they sit on, and how many plates they stand above it (tree's and
# gnarled_tree's numbers).
LEAVES = {
    "2417": ([-48.0, 0.0, -68.0, 48.0, 8.0, 48.0], 1),
    "2423": ([-26.0, 0.0, -68.0, 26.0, 8.0, 10.0], 1),
    "6255": ([-46.061, -27.45, -33.194, 45.979, 16.0, 54.95], 2),
    "15279": ([-8.0, -63.993, -19.391, 8.0, 0.0, 14.729], 0),
}


def on_stud(m, part, color, cx, cy, z, rot=0):
    """Puts a plant part on the stud whose middle is (cx, cy) and whose top is at z."""
    body, above = LEAVES[part]
    turn = frame.to_ldraw_rotation(frame.quarter_turns(rot))
    lo, _ = frame.box_in_dsl(frame.transform_box(turn, (0, 0, 0), body))
    return m.add(part, color, at=(cx + lo[0], cy + lo[1], z + above + lo[2]), rot=rot)


def tall_tree(m, trunk, thin, crown):
    """A broad-leaved tree taller than the house, in its own frame (tree, grown tall): flared roots (gnarled_tree's), a
    trunk of `trunk` round 2 x 2 bricks, jumpers that bring one stud to its middle, `thin` round 1 x 1 bricks, then the
    crown: leaves stacked straight up on the stud of the one under, each turned a quarter further, small ones at the
    bottom and top and `crown` big ones between, so it swells to an oval, in two greens, and a plant on top."""
    bark = "Reddish Brown"
    with m.step("Roots"):
        # A corner-round brick bulges back right at rot=0, a quarter on with each turn.
        for (rx, ry), rot in (((0, 0), 2), ((2, 0), 3), ((2, 2), 0), ((0, 2), 1)):
            m.add("3063b", bark, at=(rx, ry, 0), rot=rot)
        m.add("3941", bark, at=(1, 1, 0))
    z = 3 + 3 * trunk
    with m.step("Trunk"):
        m.stack("3941", bark, at=(1, 1, 3), count=trunk)
        # Two jumpers along y over the 2 x 2's studs, and one across them: its stud is in the middle.
        m.add("3794b", bark, at=(1, 1, z), rot=0)
        m.add("3794b", bark, at=(2, 1, z), rot=0)
        m.add("3794b", bark, at=(1, 1.5, z + 1), rot=1)
    z += 2
    with m.step("Trunk, top"):
        m.stack("3062b", bark, at=(1.5, 1.5, z), count=thin)
    z += 3 * thin
    layers = ["2423"] * 2 + ["2417"] * crown + ["2423"] * 3
    actions = []
    for i, part in enumerate(layers):
        color = ("Green", "Bright Green")[i % 2] if part == "2417" else "Green"
        actions.append(
            (
                1,
                lambda part=part, color=color, i=i, z=z + i: on_stud(
                    m, part, color, 2, 2, z, rot=i % 4
                ),
            )
        )
    actions.append(
        (1, lambda: on_stud(m, "6255", "Bright Green", 2, 2, z + len(layers)))
    )
    in_steps(m, "Crown", actions)


def snowman(m):
    """A snowman: a round brick and a dome, a round head, and a hat."""
    with m.step("Snowman"):
        m.add("3941", "White", at=(0, 0, 0), id="body")
        m.add("30367c", "White", at=(0, 0, 3), id="belly")
        m.add("3062b", "White", at=(0.5, 0.5, 6), id="head")
        m.add("6141", "Black", at=(0.5, 0.5, 9), id="hat/brim")
        m.add("3062b", "Black", at=(0.5, 0.5, 10), id="hat")


def dress(m, title, cells, z, rng, n, blooms, wild=False):
    """Dots the ground among `cells` (free studs at height z): n spots, none beside another, each a tuft of leaves
    (where there's room all round), a flower on a stem, a pebble, or a patch of paler grass, so no wide stretch of
    bare studs is left. Wild, it's long grass and tall flowers on stems of round bricks instead of pebbles and
    patches."""
    free = set(cells)
    spots = []
    for c in rng.sample(sorted(free), len(free)):
        if len(spots) < n and all(
            max(abs(c[0] - o[0]), abs(c[1] - o[1])) > 1 for o in spots
        ):
            spots.append(c)
    actions, tufts = (
        [],
        [],
    )  # the tufts go in last: their leaves spread over the studs round them
    for x, y in spots:
        roomy = all((x + i, y + j) in free for i in range(-2, 3) for j in range(-2, 3))
        patch = all((x + i, y + j) in free for i in (0, 1) for j in (0, 1))
        if wild:
            kind = rng.choice(["grass"] * 2 + ["tall flower"] * 3)
        else:
            kind = rng.choice(
                ["tuft"] * roomy * 2 + ["flower"] * 2 + ["pebble"] + ["patch"] * patch
            )
        if kind == "tuft":
            green = rng.choice(("Green", "Bright Green"))
            tufts.append(
                (
                    1,
                    lambda x=x, y=y, green=green: on_stud(
                        m, "6255", green, x + 0.5, y + 0.5, z
                    ),
                )
            )
        elif kind == "flower":
            bloom = rng.choice(blooms)
            actions.append(
                (
                    2,
                    lambda x=x, y=y, bloom=bloom: [
                        m.add("6141", "Green", at=(x, y, z)),
                        m.add("33291", bloom, at=(x - 0.1, y - 0.1, z + 1)),
                    ],
                )
            )
        elif kind == "grass":
            blade = rng.choice(("Bright Green", "Lime", "Olive Green"))
            actions.append(
                (
                    1,
                    lambda x=x, y=y, blade=blade: on_stud(
                        m, "15279", blade, x + 0.5, y + 0.5, z
                    ),
                )
            )
        elif kind == "tall flower":
            bloom, stem = rng.choice(blooms), rng.randint(1, 2)
            actions.append(
                (
                    stem + 1,
                    lambda x=x, y=y, bloom=bloom, stem=stem: [
                        m.stack("3062b", "Green", at=(x, y, z), count=stem),
                        m.add("33291", bloom, at=(x - 0.1, y - 0.1, z + 3 * stem)),
                    ],
                )
            )
        elif kind == "pebble":
            actions.append(
                piece(
                    m,
                    "98138",
                    rng.choice(("Light Bluish Gray", "Dark Bluish Gray")),
                    (x, y, z),
                )
            )
        else:
            actions.append(piece(m, "14769", "Bright Green", (x, y, z)))
    in_steps(m, title, actions + tufts)


@model(
    knobs=[
        "season",
        "roof_style",
        "wall_finish",
        "garden",
        "lights",
        "roof",
        "weathering",
        "seed",
    ]
)
def cottage(
    width: Annotated[int, Range(14, 20), Step(2)] = 16,
    depth: Annotated[int, Range(10, 12), Step(2)] = 10,
    wall_color: Color = "White",
    roof_color: Color = "Dark Red",
    season: Literal["summer", "winter"] = "summer",
    roof_style: Literal["tiles", "slate", "shingles", "thatch"] = "tiles",
    wall_finish: Literal["plaster", "brick", "timber", "siding"] = "plaster",
    garden: Literal["tidy", "wild"] = "tidy",
    lights: bool = False,
    chimney: bool = True,
    roof: bool = True,
    weathering: Annotated[int, Range(0, 30)] = 0,
    seed: Annotated[int, Range(1, 99)] = 1,
    baseplate: bool = True,
):
    """A stone-and-plaster cottage with a garden, on a green 32 x 32 baseplate.
    width: how many studs wide the house is
    depth: how many studs deep the house is
    wall_color: the color of the plaster
    roof_color: the color of the roof
    season: summer (flowers) or winter (snow on the roof and the garden, a snowman, wreaths on the porch)
    roof_style: what the roof is made of: tiles, slate, shingles, or straw thatch with a rounded ridge
    wall_finish: what covers the walls above the stone: plaster, brick, half-timbering, or wooden siding
    garden: tidy, or wild (overgrown, with long grass, tall flowers, roses climbing the fence, and a blossom tree)
    lights: evening: the windows glow, and the porch lantern and the lamps are lit
    chimney: add a stone chimney on the right end, with the fireplace inside
    roof: put the roof on (off: it's lifted off, so the rooms show)
    weathering: how weathered the walls are, in percent: old stone through the plaster, patched bricks, worn boards
    seed: which arrangement of the old stones and the garden (Shuffle picks another)
    baseplate: stand the cottage in its garden on a green 32 x 32 baseplate (off: the house alone, on the table)
    """
    m = Model()
    winter = season == "winter"
    ground = BASEPLATE if baseplate else 0  # the height everything stands on
    F = ground + 4  # the floor plates' top: a brick of plinth and a plate of floor
    T = F + 22  # the walls' top
    rng = random.Random(seed)

    # The palette: plaster over stone, a plinth of darker stone, a timber band, dark green door and shutters.
    plinth, stone, trim, green = (
        "Dark Bluish Gray",
        "Tan",
        "Reddish Brown",
        "Dark Green",
    )
    snow = "White"
    # Over the stone: plaster, brick in two tones, plaster between dark timbers, or boards of natural wood. The gables
    # match.
    brick, patch, timber, boards = (
        "Nougat",
        "Medium Nougat",
        "Dark Brown",
        "Medium Nougat",
    )
    beams = (
        timber if wall_finish == "timber" else wall_color
    )  # the sill and hood courses
    gable_pieces, gable_color = {
        "plaster": (BRICKS, wall_color),
        "brick": (EMBOSSED, brick),
        "timber": (BRICKS, wall_color),
        "siding": (LOGS, boards),
    }[wall_finish]
    # The roof's main color: slate is dark gray and thatch straw, tiles and shingles take roof_color; every roof is
    # under snow in winter.
    roofing = (
        snow
        if winter
        else {"slate": "Dark Bluish Gray", "thatch": "Tan"}.get(roof_style, roof_color)
    )

    # The house's corners: off center on the baseplate, toward the back left, so the front garden is deep.
    x0, y0 = 4, 14
    x1, y1 = x0 + width - 1, y0 + depth - 1
    # Two rooms: the kitchen with the bed corner on the left, the living room (6 studs inside, 5 in the narrowest
    # house) on the right, where the fireplace backs onto the chimney.
    px = x1 - 7 if width >= 16 else x1 - 6
    dx = px + 1  # the front door, into the living room
    bx = x0 + 1  # the bay window, 6 wide, in front of the kitchen
    cy = (
        y0 + (depth - 4) // 2
    )  # the chimney and the fireplace, 4 wide, in the middle of the right end
    kitchen_window = (x0 + 1 + px - 1) // 2
    living_window = (px + 1 + x1 - 1) // 2
    units = [
        # name, face, first of its four cells, shutters
        ("left", "-x", (x0, cy), True),
        ("back/1", "+y", (kitchen_window - 1, y1), False),
        ("back/2", "+y", (living_window - 1, y1), False),
    ]

    def unit_cells(face, cell, middle_only=False):
        ux, uy = cell
        span = range(1, 3) if middle_only else range(4)
        if face in ("-y", "+y"):
            return {(ux + i, uy) for i in span}
        return {(ux, uy + i) for i in span}

    door = {(dx + i, y0) for i in range(4)}
    bay = {(bx + i, y0) for i in range(6)}
    # A unit with shutters takes its four cells from the band up; one without, its middle two from the sill course
    # up; both take all four for the hood.
    shuttered = set().union(*(unit_cells(f, c) for _, f, c, sh in units if sh))
    boxed = set().union(
        *(unit_cells(f, c, middle_only=True) for _, f, c, sh in units if not sh)
    )
    hoods = set().union(*(unit_cells(f, c) for _, f, c, _ in units))
    corners = {(x0, y0), (x1, y0), (x0, y1), (x1, y1)}

    def face_of(c):
        if c[1] == y0:
            return "-y"
        if c[1] == y1:
            return "+y"
        return "-x" if c[0] == x0 else "+x"

    ring = {(x, y) for x in range(x0, x1 + 1) for y in (y0, y1)}
    ring |= {(x, y) for y in range(y0 + 1, y1) for x in (x0, x1)}

    # The chimney: a stone stack outside the right end, 4 wide and 2 out, which rises with the walls a course a step,
    # steps in at the shoulder under the roof, and gets its top after the roof (banded_chimney, split by course).
    # The roof's front and back edges, a stud out from the walls, and how many courses of slopes it takes to meet.
    ya, yb = y0 - 1, y1 + 2
    roof_courses = (yb - ya - 4) // 2 + 1
    ridge_top = T + 3 + 3 * roof_courses + 3
    stack = []  # (z, how many parts, the function that adds them), from the bottom up
    if chimney:
        cx = x1 + 1
        numbers = itertools.count(1)

        def put(part, color, at, rot=0):
            m.add(part, color, at=at, rot=rot, id=f"chimney/{next(numbers)}")

        def body(z, r):
            """A course of the 4 x 2 body: 1 x 2 bricks, each course turned from the one below so they bond."""
            if r % 2 == 0:
                return lambda: [
                    put("98283", stone, (cx + v, cy + u, z))
                    for u in (0, 2)
                    for v in (0, 1)
                ]
            return lambda: [
                put("98283", stone, (cx, cy + u, z), rot=1) for u in range(4)
            ]

        def shoulder(z):
            return lambda: [
                put("3040b", stone, (cx + v, cy + 2 * k, z), rot=3 - 2 * k)
                for k in (0, 1)
                for v in (0, 1)
            ]

        def column(z, r):
            """A course of the 2 x 2 stack over the shoulder."""
            if r % 2:
                return lambda: [
                    put("98283", stone, (cx + k, cy + 1, z)) for k in (0, 1)
                ]
            return lambda: [
                put("98283", stone, (cx, cy + 1 + k, z), rot=1) for k in (0, 1)
            ]

        def cap(z):
            return lambda: [
                put("3020", snow if winter else plinth, (cx, cy, z)),
                put("4589", "Reddish Brown", (cx + 1, cy + 1, z + 1)),
                put("4589", "Reddish Brown", (cx + 1, cy + 2, z + 1)),
            ]

        # The body up to the roof's frame, a band of dark stone every three courses.
        z, r = ground, 0
        while z + 3 <= T + 3:
            stack.append((z, 4, body(z, r)))
            z, r = z + 3, r + 1
            if r % 3 == 0 and z + 3 <= T + 3:
                stack.append((z, 1, lambda z=z: put("3020", plinth, (cx, cy, z))))
                z += 1
        # The shoulder: slopes along the wall step it in to the 2 x 2 stack in the middle, which rises a brick or two
        # past the ridge to a cap and two pots.
        stack.append((z, 4, shoulder(z)))
        z, r = z + 3, 0
        while z < ridge_top + 3:
            stack.append((z, 2, column(z, r)))
            z, r = z + 3, r + 1
        stack.append((z, 3, cap(z)))
    laid = 0  # how many of the chimney's layers are in

    def chimney_to(top):
        """The chimney's layers that start below `top`, not yet laid, as in_steps actions."""
        nonlocal laid
        out = []
        while laid < len(stack) and stack[laid][0] < top:
            out.append(stack[laid][1:])
            laid += 1
        return out

    # The walls course by course: (z, family, color, the cells left open). The plinth and the two stone courses are
    # embossed brickwork; then the timber band, the sill course, the plaster, the hood course, and a last course of
    # plaster. Openings: the door, the bay, the window units, the stone corners (quoins), and the old stones.
    plaster_rows = [F + 9, F + 12, F + 15, F + 19]
    upper = {"plaster": Brick, "brick": "stone", "timber": Brick, "siding": "logs"}[
        wall_finish
    ]
    upper_color = {"brick": brick, "siding": boards}.get(wall_finish, wall_color)
    courses = [
        (ground, "stone", plinth, set()),
        (F, Plate, plinth, set()),
        (F + 1, "stone", stone, door),
        (F + 4, "stone", stone, door),
        (F + 7, Plate, trim, door | shuttered),
        (F + 8, Plate, beams, door | bay | shuttered | boxed),
    ]
    # Inside, a clock hangs on the front wall of the living room, beside the door, when there's a stud of wall for it.
    clock = {(x1 - 2, y0)} if dx + 4 <= x1 - 2 else set()
    courses += [
        (
            z,
            upper,
            upper_color,
            door
            | bay
            | shuttered
            | boxed
            | corners
            | (clock if z == F + 12 else set()),
        )
        for z in plaster_rows[:3]
    ]
    courses += [(F + 18, Plate, beams, door | bay | hoods)]
    courses += [(F + 19, upper, upper_color, corners)]

    # Half-timbering: posts every fourth stud along each wall, and between each two a brace stepping across the
    # courses under the hood course (its ends in the first and third), leaning one way then the other; the sill and
    # hood courses are the rails, with plain plaster over the hood course. A timber is a dark brick in its cell.
    timbers = {}
    if wall_finish == "timber":
        for z, *_, gaps in courses:
            if z not in plaster_rows:
                continue
            r = plaster_rows.index(z)
            cells = set()
            for c in ring - gaps - corners:
                i = c[0] - x0 if c[1] in (y0, y1) else c[1] - y0
                bay_i, k = divmod(i, 4)
                if r < 3 and (
                    k == 0 or (r != 1 and k == (1 + r if bay_i % 2 == 0 else 3 - r))
                ):
                    cells.add(c)
            timbers[z] = cells

    # Old stone showing through the plaster: single cells, never at a corner or an opening, never beside another old
    # stone, and never over one or an opening, so the course over it bridges it like any other brick. None behind the
    # porch and the bay (their posts and roofs stand against the wall), behind the chimney or over the woodpile, or
    # beside a corner on the side its quoin's long tile reaches along: the front corners' long tiles are always on the
    # ends (the bay and a porch post stand beside them); the back ones' take turns.
    porch = {(x, y0) for x in range(dx - 1, dx + 5)}
    no_stone = porch | bay | clock | {(x1, cy + i) for i in range(-1, 4)}
    no_stone |= {(x0, y0 + 1), (x1, y0 + 1)}
    old_stones = {}
    below = set()
    for z, _, _, gaps in courses:
        if z not in plaster_rows:
            continue
        if plaster_rows.index(z) % 2 == 0:
            beside = {(x0 + 1, y1), (x1 - 1, y1)}
        else:
            beside = {(x0, y1 - 1), (x1, y1 - 1)}
        if z == plaster_rows[-1]:
            beside |= hoods  # the hoods' tiles stand in front of the last course's foot
        picked = set()
        for c in sorted(ring - gaps - no_stone - beside - timbers.get(z, set())):
            beside_it = {
                (c[0] - 1, c[1]),
                (c[0] + 1, c[1]),
                (c[0], c[1] - 1),
                (c[0], c[1] + 1),
            }
            if (
                rng.random() * 100 < weathering
                and c not in below
                and not beside_it & picked
            ):
                picked.add(c)
        old_stones[z] = picked
        below = (gaps - corners) | picked

    sides = (
        ("back wall", "back"),
        ("left wall", "left"),
        ("right wall", "right"),
        ("front wall", "front"),
    )
    on_side = {
        "back": lambda c: c[1] == y1,
        "front": lambda c: c[1] == y0,
        "left": lambda c: c[0] == x0 and y0 < c[1] < y1,
        "right": lambda c: c[0] == x1 and y0 < c[1] < y1,
    }

    def course(i, title):
        """One course of the walls, a wall a step, the far side first, with the quoins and old stones of that wall and
        the chimney's layers that rise beside it."""
        z, kind, color, gaps = courses[i]
        old = old_stones.get(z, set())
        posts = timbers.get(z, set())
        quoins = z in plaster_rows
        for name, side in sides:
            # About how many bricks it lays: a few long ones a run between the openings.
            cells = sorted(
                c for c in ring if on_side[side](c) and c not in gaps | old | posts
            )
            bricks = sum(
                # Brick and siding courses take shorter pieces than plaster's.
                (
                    n // 2 + 1
                    if kind in ("stone", "logs") and z in plaster_rows
                    else n // 3 + 1
                )
                for _, n in runs(cells, "x" if side in ("back", "front") else "y")
            )
            if kind in ("stone", "logs"):
                lay = lambda side=side: stone_course(
                    m,
                    x0,
                    y0,
                    z,
                    width,
                    depth,
                    color,
                    sides=(side,),
                    openings=gaps | old,
                    pieces=LOGS if kind == "logs" else EMBOSSED,
                )
            else:
                lay = lambda side=side: m.wall(
                    x0,
                    y0,
                    z,
                    length=width,
                    depth=depth,
                    sides=side,
                    color=color,
                    openings=gaps | old | posts,
                    kind=kind,
                    sizes=WALL_BRICKS if kind is Brick else None,
                )
            actions = [(bricks, lay)]
            for c in sorted(filter(on_side[side], posts)):
                actions.append(piece(m, "3005", timber, (*c, z)))
            for c in sorted(filter(on_side[side], old)):
                face = face_of(c)

                def stone_cell(c=c, face=face):
                    m.add("87087", wall_color, at=(*c, z), rot=STUD_ROT[face])
                    face_tile(m, stone, c, face, z)

                if wall_finish == "brick":
                    actions.append(piece(m, "3005", patch, (*c, z)))  # a patched brick
                elif wall_finish == "siding":
                    actions.append(
                        piece(m, "3005", "Dark Tan", (*c, z))
                    )  # a worn board
                elif wall_finish == "timber":
                    actions.append(
                        piece(m, "3005", stone, (*c, z))
                    )  # stone where the plaster's gone
                else:
                    actions.append((2, stone_cell))
            if quoins:
                r = plaster_rows.index(z)
                for c in sorted(filter(on_side[side], corners)):
                    # A corner belongs to the front or back wall's step.
                    def quoin(c=c):
                        m.add("4733", stone, at=(*c, z))
                        across = "-x" if c[0] == x0 else "+x"
                        along = "-y" if c[1] == y0 else "+y"
                        # The bay stands beside the front left corner, and a porch post may beside the right one.
                        long_front = r % 2 == 0 and c not in ((x0, y0), (x1, y0))
                        face_tile(
                            m,
                            stone,
                            c,
                            along,
                            z,
                            2 if long_front else 1,
                            back=c[0] == x1,
                        )
                        face_tile(
                            m,
                            stone,
                            c,
                            across,
                            z,
                            1 if long_front else 2,
                            back=c[1] == y1,
                        )

                    actions.append((3, quoin))
            if side == "right":
                actions += chimney_to(z + (1 if kind is Plate else 3))
            in_steps(m, f"{title}: {name}", actions)

    def bay_base(z, kind, color):
        if kind == "stone":
            stone_course(m, bx, y0 - 3, z, 6, 3, color)
        else:
            m.wall(bx, y0 - 3, z, length=6, depth=3, color=color, kind=kind)

    # Built the way an instruction book builds it: a few parts a step, from the baseplate up. Off the baseplate, the
    # house stands alone on the table, without its garden.
    grass = snow if winter else "Bright Green"
    lawn_x = dx + 4  # right of the porch steps, in front of the porch
    sx = (
        dx - 5
    )  # where the path leaves the gate; it ends at the steps, 6 studs to the right
    if baseplate:
        with m.step("Baseplate"):
            m.add("3811", "Green", at=(0, 0, 0), id="base")
        # The ground on four levels: the baseplate; the path a plate up, curving from the gate to the porch steps; a
        # lawn two plates up with a wandering edge; and a mound at the back right. A rock in the back left corner.
        lawn_top = m.use(
            "organic_ground_outline",
            x=lawn_x,
            y=1,
            z=ground,
            length=31 - lawn_x,
            depth=10,
            seed=seed,
            grass=grass,
            title="Lawn",
        )
        mound_top = m.use(
            "contour_terraces",
            x=x1 + 2,
            y=y1 + 1,
            z=ground,
            length=31 - x1 - 1,
            depth=32 - y1 - 1,
            terraces=1,
            seed=seed,
            grass=grass,
            title="Mound",
        )
        path = m.submodel(
            "Path",
            lambda s: s.use(
                "curved_garden_path",
                x=0,
                y=0,
                z=0,
                legs=(1, 0, 1),
                turns="RL",
                color="Tan",
                title="Path",
            ),
        )
        with m.step("Path"):
            m.place(path, at=(sx, 1, ground), rot=1, id="path/1")
        m.use("rock_outcrop", x=0, y=26, z=ground, height=2, seed=seed, title="Rock")

    # The plinth: a course of dark stone under the house, the bay, and the porch, then the floor across it.
    course(0, "Plinth")
    with m.step("Plinth: bay"):
        bay_base(ground, "stone", plinth)
    with m.step("Plinth: porch"):
        stone_course(m, dx - 1, y0 - 3, ground, 6, 3, plinth)
    in_steps(
        m,
        "Floor",
        [
            (
                1 + (width > 16),
                lambda y=y: m.row(
                    Plate, plinth, at=(x0, y, ground + 3), length=width, width=2
                ),
            )
            for y in range(y0, y1, 2)
        ],
    )
    with m.step("Floor: bay"):
        bay_base(ground + 3, Plate, plinth)
    with m.step("Floor: porch"):
        m.fill(Plate, plinth, at=(dx - 1, y0 - 3, ground + 3), size=(6, 3))

    # The rooms: what stands where (x, y, and the cells it covers each way), so the floors are finished round it. The
    # kitchen has its counter along the left wall and the bed along the back, or, when it's narrow, up the partition;
    # the living room has the fireplace on the chimney wall, the armchair and a lamp along the back, a rug between.
    kx0, kx1 = x0 + 1, px - 1  # the kitchen's first and last cells along x
    lx0, lx1 = px + 1, x1 - 1  # the living room's
    ry0, ry1 = y0 + 1, y1 - 1
    wide = kx1 - kx0 + 1 >= 6
    bed = (kx0, ry1 - 1, 6, 2) if wide else (kx1 - 1, ry1 - 5, 2, 6)
    units_n = min(4, ((ry1 - 1 if wide else ry1 + 1) - ry0) // 2)
    furniture = {
        "fireplace": (x1 - 2, cy, 2, 4),
        "armchair": (lx0, ry1 - 1, 4, 2),
        "rug": (x1 - 5, cy, 3, 3),
        "firewood": (x1 - 2, cy - 1, 2, 1),
        "bed": bed,
        "counter": (
            kx0,
            ry0,
            3,
            2 * units_n,
        ),  # its doors stand out into the third column
    }
    lamp = lx1 - lx0 + 1 >= 6
    if lamp:
        furniture["lamp"] = (x1 - 2, ry1 - 1, 2, 2)
    if lights:
        furniture["kitchen lamp"] = (
            bx + 3,
            ry0,
            2,
            2,
        )  # behind the bay, so it glows through its windows
    table = kx1 - kx0 + 1 >= 7
    cells = set()
    for fx, fy, w, d in furniture.values():
        cells |= {(fx + i, fy + j) for i in range(w) for j in range(d)}
    if table:
        # The set turned to face +x: the table's foot and its two chairs stand on studs; the floor runs round them.
        tx, ty = kx0 + 3, ry0 + 1
        cells |= {(tx + 1 + i, ty + 1 + j) for i in (0, 1) for j in (0, 1)} | {
            (tx + 2, ty),
            (tx + 1, ty + 3),
        }

    # Floors: old boards in the living room, a checker of tiles in the kitchen.
    floor = []
    for y in range(ry0, ry1 + 1):
        x = lx0
        while x <= lx1:
            if (x, y) in cells:
                x += 1
                continue
            run = 0
            while x + run <= lx1 and (x + run, y) not in cells:
                run += 1
            for n in staggered(run, (4, 3, 2, 1), y):
                wood = rng.choice(("Reddish Brown", "Medium Nougat"))
                floor.append(piece(m, TILES[n], wood, (x, y, F), rot=1))
                x += n
    in_steps(m, "Floor: living room boards", floor)
    floor = []
    for y in range(ry0, ry1 + 1):
        for x in range(kx0, kx1 + 1):
            if (x, y) not in cells:
                c = "White" if (x + y) % 2 else "Light Bluish Gray"
                floor.append(piece(m, "3070b", c, (x, y, F)))
    in_steps(m, "Floor: kitchen tiles", floor)

    course(1, "Walls, sole plate")
    with m.step("Walls, sole plate: bay"):
        bay_base(F, Plate, plinth)
    in_steps(
        m,
        "Porch: landing",
        [
            piece(m, "3070b", "Light Bluish Gray", (dx - 1 + i, y0 - 3 + j, F))
            for j in range(3)
            for i in range(6)
            if i not in (0, 5)  # the posts' studs, and the pots' between them
        ],
    )

    # The door in its frame and each window are sub-assemblies: built apart, once, then put in the wall.
    door_unit = m.submodel("Door", door_in_frame, color=green)
    course(2, "Walls, row 1")
    with m.step("Walls, row 1: bay"):
        bay_base(F + 1, "stone", stone)
    with m.step("Door"):
        m.place(door_unit, at=(dx, y0, F + 1), rot=3, id="door")
    course(3, "Walls, row 2")
    with m.step("Walls, row 2: bay"):
        bay_base(F + 4, "stone", stone)

    # The rooms, furnished while the walls are two bricks high.
    m.use(
        "hearth_fireplace",
        x=x1 - 2,
        y=cy,
        z=F,
        facing="-x",
        flue=4,
        color="Light Bluish Gray",
        title="Fireplace",
        id="fireplace",
    )
    m.use(
        "armchair_and_sofa",
        x=lx0,
        y=ry1 - 1,
        z=F,
        facing="-y",
        color="Dark Red",
        title="Armchair",
    )
    if lamp:
        lit = {"shade": "Trans-Yellow"} if lights else {}
        m.use(
            "floor_lamp_and_candle", x=x1 - 2, y=ry1 - 1, z=F, title="Floor lamp", **lit
        )
    if lights:
        m.use(
            "floor_lamp_and_candle",
            x=bx + 3,
            y=ry0,
            z=F,
            shade="Trans-Yellow",
            title="Floor lamp",
        )
    m.use(
        "floor_rug", x=x1 - 5, y=cy, z=F, shape="round", color="Dark Red", title="Rug"
    )
    m.use(
        "woodpile",
        x=x1 - 2,
        y=cy - 1,
        z=F,
        length=2,
        rows=2,
        seed=seed,
        title="Firewood",
    )
    m.use(
        "kitchen_counter",
        x=kx0,
        y=ry0,
        z=F,
        units=("stove", "sink", "cabinet", "cabinet")[:units_n],
        facing="+x",
        title="Counter",
    )
    m.use(
        "figure_bed",
        x=bed[0],
        y=bed[1],
        z=F,
        facing="+x" if wide else "-y",
        quilt="Dark Blue",
        title="Bed",
        id="bed",
    )
    if table:
        m.use(
            "table_and_chairs",
            x=kx0 + 3,
            y=ry0 + 1,
            z=F,
            facing="+x",
            chairs=2,
            title="Table",
            id="table",
        )
    m.use(
        "interior_arch_doorway",
        x=px,
        y=ry0,
        z=F,
        length=depth - 2,
        axis="y",
        gap=1,
        rows=7,
        color="White",
        arch=stone,
        title="Partition",
    )

    course(4, "Walls, band")
    with m.step("Walls, band: bay"):
        bay_base(F + 7, Plate, trim)
    course(5, "Walls, sill course")
    if lights:
        lit_bay(m, bx, y0, F + 8, "White", trim, roofing, "Bay window")
    else:
        m.use(
            "bay_window",
            x=bx,
            y=y0,
            z=F + 8,
            width=6,
            color="White",
            trim=trim,
            roof=roofing,
            title="Bay window",
        )
    course(6, "Walls, row 3")
    course(7, "Walls, row 4")
    if clock:
        m.use(
            "wall_picture_and_clock",
            x=x1 - 2,
            y=y0,
            z=F + 12,
            kind="clock",
            facing="+y",
            color=wall_color,
            title="Clock",
        )
    course(8, "Walls, row 5")
    flowers = ("Dark Pink", "Bright Light Orange") if not winter else ("Red", "Red")
    window_units = {}
    for name, face, cell, shutters in units:
        if shutters not in window_units:
            window_units[shutters] = m.submodel(
                "Window",
                window_unit,
                shutters=shutters,
                trim=trim,
                wall=wall_color,
                shutter=green,
                flowers=flowers,
                **({"lights": True} if lights else {}),
            )
        with m.step(f"Window, {name.replace('/', ' ')}"):
            place_on_wall(
                m,
                window_units[shutters],
                face,
                cell,
                4,
                F + 7 if shutters else F + 8,
                id=f"window/{name}",
            )
    course(9, "Walls, hood course")
    course(10, "Walls, row 6")

    # The porch: four posts on the landing, a plate roof, and slopes; two rounded steps down to the path.
    with m.step("Porch: steps"):
        m.add("3020", "Light Bluish Gray", at=(dx, y0 - 5, ground), rot=1)
        for i in range(4):
            m.add("11477", "Light Bluish Gray", at=(dx + i, y0 - 5, ground + 1), rot=3)
    posts = ((dx - 1, y0 - 3), (dx + 4, y0 - 3), (dx - 1, y0 - 1), (dx + 4, y0 - 1))
    for i, (qx, qy) in enumerate(posts):
        with m.step("Porch: posts"):
            m.stack("3062b", "White", at=(qx, qy, F), count=5, id=f"porch/post/{i + 1}")
            m.add("6141", "White", at=(qx, qy, F + 15))
    # The porch roof's plates, and under their front, between the posts, a beam with studs on its face that carries
    # the house's nameplate, or in winter two wreaths.
    with m.step("Porch: roof plates and beam"):
        m.add("3795", trim, at=(dx - 1, y0 - 3, F + 16), rot=1)
        m.add("3666", trim, at=(dx - 1, y0 - 1, F + 16), rot=1)
        m.add("30414", trim, at=(dx, y0 - 3, F + 13), rot=3, id="porch/beam")
    with m.step("Porch: nameplate" if not winter else "Porch: wreaths"):
        if winter:
            for i in (1, 2):
                m.add(
                    "33291",
                    "Green",
                    at=(dx + i - 0.1, y0 - 3.4, F + 13.25),
                    rot=Rot(x=90),
                    id=f"wreath/{i}",
                )
        else:
            m.add(
                "2431",
                stone,
                at=(dx, y0 - 3.4, F + 13.5),
                rot=Rot(y=-90, z=90),
                id="nameplate",
            )
    if lights:
        # A lantern hangs under the porch roof beside the door, over the pot: a cap, a glowing round brick, a plate,
        # and a cone (wall_lamp_bracket's lantern).
        with m.step("Porch: lantern"):
            for part, color, dz in (
                ("6141", "Black", 1),
                ("3062b", "Trans-Yellow", 4),
                ("6141", "Black", 5),
                ("4589", "Black", 7),
            ):
                m.add(
                    part,
                    color,
                    at=(dx - 1, y0 - 2, F + 16 - dz),
                    id=f"lantern/{part}-{dz}",
                )
    with m.step("Porch: pots"):
        for i, px_ in enumerate((dx - 1, dx + 4)):
            m.use(
                "potted_plant",
                x=px_,
                y=y0 - 2,
                z=F,
                kind="small",
                pot="Dark Orange",
                leaves="Green",
                bloom="Red" if winter else ("Yellow", "Dark Pink")[i],
                title="Porch: pots",
            )
    with m.step("Porch: roof"):
        for i in range(3):
            m.add("3039", roofing, at=(dx - 1 + 2 * i, y0 - 3, F + 17), rot=3)
        m.add("3009", roofing, at=(dx - 1, y0 - 1, F + 17), rot=1)

    # The wall tops, tiled smooth but for a plate at each corner, where the roof's frame locates (lift_off_roof_seat).
    tiles = (1, 2, 3, 4, 6, 8)
    in_steps(
        m,
        "Roof seat: tiles",
        [
            (
                3,
                lambda: m.row(
                    Tile, wall_color, at=(x0 + 1, y1, T), length=width - 2, sizes=tiles
                ),
            ),
            (
                3,
                lambda: m.row(
                    Tile,
                    wall_color,
                    at=(x0, y0 + 1, T),
                    length=depth - 2,
                    axis="y",
                    sizes=tiles,
                ),
            ),
            (
                3,
                lambda: m.row(
                    Tile,
                    wall_color,
                    at=(x1, y0 + 1, T),
                    length=depth - 2,
                    axis="y",
                    sizes=tiles,
                ),
            ),
            (
                3,
                lambda: m.row(
                    Tile, wall_color, at=(x0 + 1, y0, T), length=width - 2, sizes=tiles
                ),
            ),
        ],
    )
    with m.step("Roof seat: corner studs"):
        for c in sorted(corners):
            m.add("3024", wall_color, at=(*c, T))

    # The roof: one sub-assembly, built on its own frame, which sits on the four corner studs and lifts off. The frame
    # stands a stud out over the front and back walls, its edge the eave's dark fascia; the gables overhang a stud
    # too, unless the chimney stands against the right one.
    L = x0 - 1 if not chimney else x0
    R = x1 + 2 if not chimney else x1 + 1
    dormer_x = dx

    # How each style lays a course: tiles and slate are 45-degree slopes, slate mottled with black ones; shingles are
    # 2-deep bricks, each course a stud back and a brick up, their studs the rows of shingles; thatch is slopes in
    # straw, the short ones of every other course in a darker straw, over a thick straw eave, under a rounded ridge.
    roof_rng = random.Random(seed + 2)
    eave = "Dark Tan" if roof_style == "thatch" and not winter else trim

    def slope_color(k, n):
        if winter or roof_style in ("tiles", "shingles"):
            return roofing
        if roof_style == "slate":
            return "Black" if roof_rng.random() < 0.25 else roofing
        return (
            "Dark Tan" if k % 2 and n < 4 else roofing
        )  # Dark Tan slopes come only up to 3 long

    def roof_course(s, k, x, n, sy, z, turn):
        """One side of course k from x, n studs long: its parts as in_steps actions."""
        actions = []
        if roof_style == "shingles":
            for size in staggered(n, COURSES, k):
                actions.append(piece(s, COURSES[size], roofing, (x, sy, z), rot=1))
                x += size
            return actions
        for size in staggered(n, SLOPES, k):
            actions.append(
                piece(s, SLOPES[size], slope_color(k, size), (x, sy, z), rot=turn)
            )
            x += size
        return actions

    def roof_on_frame(s):
        with s.step("Frame"):
            s.row(Plate, eave, at=(L, ya, T + 1), length=R - L, width=2)
            s.row(Plate, eave, at=(L, yb - 2, T + 1), length=R - L, width=2)
            s.row(
                Plate,
                eave,
                at=(L, ya + 2, T + 1),
                length=yb - ya - 4,
                axis="y",
                width=2,
            )
            s.row(
                Plate,
                eave,
                at=(R - 2, ya + 2, T + 1),
                length=yb - ya - 4,
                axis="y",
                width=2,
            )
        with s.step("Frame, second layer"):
            s.row(Plate, eave, at=(L, ya, T + 2), length=yb - ya, axis="y", width=2)
            s.row(Plate, eave, at=(R - 2, ya, T + 2), length=yb - ya, axis="y", width=2)
            s.row(Plate, eave, at=(L + 2, ya, T + 2), length=R - L - 4, width=2)
            s.row(Plate, eave, at=(L + 2, yb - 2, T + 2), length=R - L - 4, width=2)
        gable = None
        for k in range(roof_courses):
            z = T + 3 + 3 * k

            for side, sy, turn in (("front", ya + k, 3), ("back", yb - 2 - k, 1)):
                actions = []
                x = L
                gap = (dormer_x, dormer_x + 4) if side == "front" and k == 0 else None
                for run_from, run_to in [(L, gap[0]), (gap[1], R)] if gap else [(L, R)]:
                    actions += roof_course(
                        s, k, run_from, run_to - run_from, sy, z, turn
                    )
                in_steps(s, f"Roof, course {k + 1}: {side}", actions)
            # The gables: plaster bricks behind the slopes at each end, a round window in the left one.
            inner = list(range(ya + k + 2, yb - 2 - k))
            if inner:
                actions = []
                for gx in (x0, x1):
                    win = gx == x0 and k in (1, 2)
                    mid = (y0 + y1) // 2
                    rows = [(gx, y) for y in inner if not (win and mid <= y <= mid + 1)]
                    # Half-timbered, the gables are plaster between dark studs, every third stud.
                    studs = [
                        c
                        for c in rows
                        if wall_finish == "timber" and (c[1] - y0) % 3 == 0
                    ]
                    actions += [piece(s, "3005", timber, (gx, c[1], z)) for c in studs]
                    rows = [c for c in rows if c not in studs]
                    for (_, start), n in runs(rows, "y"):
                        for size in lengths(n, gable_pieces):
                            actions.append(
                                piece(
                                    s, gable_pieces[size], gable_color, (gx, start, z)
                                )
                            )
                            start += size
                    if win and k == 1:
                        actions.append(
                            piece(s, "7276", green, (x0, mid, z), id="window/gable")
                        )
                in_steps(s, f"Roof, course {k + 1}: gables", actions)
            if k == 0 and lights:
                gable = lit_dormer(s, dormer_x, ya, z, "White", roofing, "Dormer")
            elif k == 0:
                gable = s.use(
                    "roof_dormer",
                    x=dormer_x,
                    y=ya,
                    z=z,
                    color="White",
                    roof=roofing,
                    title="Dormer",
                )
            if k == 1 and gable:
                gable()
        top = T + 3 + 3 * roof_courses
        if roof_style == "thatch":
            # A rounded ridge: bricks 2 x 4 with a curved top, and two 2 x 1 ones for the last two studs.
            ridge = []
            for x in range(L, R, 4):
                if x + 4 <= R:
                    ridge.append(
                        piece(
                            s,
                            "6192b",
                            "Dark Tan" if not winter else snow,
                            (x, ya + roof_courses, top),
                            1,
                        )
                    )
                else:
                    ridge += [
                        piece(
                            s,
                            "5841",
                            "Dark Tan" if not winter else snow,
                            (x + i, ya + roof_courses, top),
                            1,
                        )
                        for i in (0, 1)
                    ]
        else:
            ridge = [
                piece(s, "3043", roofing, (x, ya + roof_courses, top), rot=1)
                for x in range(L, R, 2)
            ]
        in_steps(s, "Roof, ridge", ridge)

    roof_unit = m.submodel("Roof", roof_on_frame)
    if roof:
        with m.step("Roof on"):
            m.place(roof_unit, at=roof_unit.box[0], id="roof")
    in_steps(m, "Chimney top", chimney_to(10_000))

    if not baseplate:
        return m

    # The garden: the fence and gate, a mailbox, beds, hedges, a tree taller than the ridge, the bench on paving,
    # rocks on the mound, the woodpile by the chimney.
    left = 1 if sx < 10 else 2
    fence_x = sx - 2 - 4 * left
    m.use(
        "garden_gate",
        x=fence_x,
        y=0,
        z=ground,
        left=left,
        right=min(4, (32 - fence_x - 4 * left - 6) // 4),
        color="White",
        title="Fence and gate",
    )
    # The mailbox inside the gate: a container box with its slotted lid, facing the street, on a wooden post.
    with m.step("Mailbox: post"):
        m.stack(
            "3062b", "Reddish Brown", at=(sx - 3, 2, ground), count=2, id="mailbox/post"
        )
        m.add("3022", "Reddish Brown", at=(sx - 4, 2, ground + 6), id="mailbox/plate")
    with m.step("Mailbox"):
        m.add("4345b", "Red", at=(sx - 4, 1.5, ground + 7), rot=3, id="mailbox")
        # The lid hangs in the box's open front: its body starts where the box's does.
        m.add("4346", "Red", at=(sx - 4, 1.5, ground + 7), rot=3, id="mailbox/lid")
    if lights:
        # In the evening, a lit lamp by the gate.
        with m.step("Lamp"):
            m.use("lamp_post", at=(sx - 1, 1, ground), id="lamp")
    # The bed under the bay keeps its winter roses in the snow.
    blooms = ("White", "Red") if winter else ("Red", "Dark Pink", "Bright Light Orange")
    m.use(
        "flower_bed",
        x=2,
        y=8,
        z=ground,
        length=6,
        depth=1,
        seed=seed,
        edge="Dark Tan",
        flowers=blooms,
        title="Flower bed, front",
    )
    # On the lawn: stepping stones from beside the porch steps to a patch of paving, the bench at its back, and a bed
    # (or a snowman) behind the stones when there's room.
    core = 31 - lawn_x - 4  # the lawn's full studs along x, from lawn_x + 2
    pave = core - 4
    m.use(
        "stepping_stones",
        x=lawn_x + 2,
        y=4,
        z=lawn_top,
        length=4,
        wander=0,
        seed=seed,
        title="Stepping stones",
    )
    m.use(
        "flagstone_paving",
        x=lawn_x + 6,
        y=4,
        z=lawn_top,
        length=pave,
        depth=2,
        seed=seed,
        title="Paving",
    )
    with m.step("Bench"):
        bench_x = max(lawn_x + 2, min(lawn_x + core - 2, lawn_x + 6 + (pave - 4) // 2))
        m.use("bench", at=(bench_x, 6, lawn_top), id="bench")
    # The stepping stone can be one 3 deep, reaching the bed's row: measure it, built apart, and leave the bed out.
    stone = m.submodel(
        "Stone",
        lambda s: s.use(
            "stepping_stones", x=0, y=0, z=0, length=4, wander=0, seed=seed
        ),
    )
    if not winter and core >= 8 and stone.box[1][1] - stone.box[0][1] <= 2:
        m.use(
            "flower_bed",
            x=lawn_x + 2,
            y=6,
            z=lawn_top,
            length=2,
            depth=1,
            seed=seed + 1,
            edge="Dark Tan",
            title="Flower bed, lawn",
        )
    if winter:
        sm = m.submodel("Snowman", snowman)
        with m.step("Snowman"):
            m.place(sm, at=(lawn_x + 2, 7, lawn_top), id="snowman")
    # Hedges along the left and the back, in lengths of 8 (their base plates come in Green up to 10 long), a rock in
    # the corner between them, the tree on the mound, and the woodpile by the chimney.
    for i in range(3):
        m.use(
            "clipped_hedge",
            x=0,
            y=2 + 8 * i,
            z=ground,
            length=8,
            facing="right",
            title="Hedge, left",
        )
    hx = 6
    while hx + 4 <= x1 + 2:
        n = min(8, (x1 + 2 - hx) // 2 * 2)
        m.use(
            "clipped_hedge",
            x=hx,
            y=30,
            z=ground,
            length=n,
            facing="back",
            title="Hedge, back",
        )
        hx += n
    # The tree on the mound: a short trunk and a tall oval crown, whose top clears the ridge by a brick or more.
    crown = ridge_top + 3 - mound_top - (3 + 6 + 2 + 6 + 5 + 2)
    tree = m.submodel("Tree", tall_tree, trunk=2, thin=2, crown=int(crown) + 1)
    lo = tree.box[0]
    # The roots' corner: the mound's middle, but no further back than keeps the crown over the baseplate.
    tx, ty = (x1 + 2 + 32) // 2 - 2, min((y1 + 1 + 32) // 2 - 2, 26)
    with m.step("Tree"):
        m.place(tree, at=(tx + lo[0], ty + lo[1], mound_top + lo[2]), id="tree")
    # Without the chimney the roof overhangs the right end a stud, so the woodpile stands clear of it.
    m.use(
        "woodpile",
        x=x1 + (1 if chimney else 2),
        y=cy - 1,
        z=ground,
        length=4,
        rows=3,
        seed=seed,
        title="Woodpile",
    )
    # The right side closed with picket fence, between the lawn and the mound.
    with m.step("Fence, right"):
        for k in range((y1 + 1 - 12) // 4):
            m.add(
                "33303",
                "White",
                at=(31, 12 + 4 * k, ground),
                rot=2,
                id=f"fence/right/{k + 1}",
            )
    if garden == "wild":
        # Roses climbing the right fence: a stem of round bricks inside it, two leaves stacked on top reaching along
        # it over the pickets, and a bloom.
        for k in range((y1 + 1 - 12) // 4):
            vy = 13 + 4 * k
            with m.step("Fence, right: roses"):
                m.stack("3062b", "Green", at=(30, vy, ground), count=2)
                on_stud(m, "2423", "Green", 30.5, vy + 0.5, ground + 6, rot=3)
                on_stud(m, "2423", "Dark Green", 30.5, vy + 0.5, ground + 7, rot=1)
                m.add(
                    "33291",
                    "Red" if not winter else "White",
                    at=(29.9, vy - 0.1, ground + 8),
                )
    # And the open ground dressed: beside the path, down the left side, right of the house, and behind it.
    taken = {
        (x, y) for x in range(x1 + 1, x1 + 6) for y in range(cy - 2, cy + 5)
    }  # the chimney and the woodpile
    taken |= {
        (tx + 2 + i, ty + 2 + j) for i in range(-5, 5) for j in range(-5, 5)
    }  # under the tree's crown
    if garden == "wild":
        taken |= {
            (x, y) for x in range(2, 5) for y in range(4, 7)
        }  # the blossom tree's foot
        taken |= {
            (x, y) for x in range(sx - 5, sx - 1) for y in range(1, 6)
        }  # tall flowers clear of the mailbox
        taken |= {(x, y) for x in (28, 29) for y in range(11, 28)}  # and of the roses
    regions = [
        (
            "front",
            [(x, y) for x in range(2, sx - 1) for y in range(4, 8)]
            + [(x, y) for x in range(2, sx - 5) for y in (1, 2, 3)],
        ),
        (
            "left side",
            [(2, y) for y in range(12, y1 - 1) if not cy - 1 <= y <= cy + 4]
            if chimney
            or garden == "tidy"  # wild long grass would stand under the left eave
            else [],
        ),  # not under the box
        (
            "right side",
            [(x, y) for x in range(x1 + 2, 30) for y in range(14, y1 + 1)]
            + [
                (x, 12) for x in range(lawn_x + 1, 30)
            ],  # clear of the eave and the quoins
        ),
        (
            "back",
            [(x, y) for x in range(8, x1) for y in range(y1 + 3, 29)],
        ),  # clear of the window boxes
    ]
    flowers = (
        ("White",) if winter else ("Red", "Yellow", "Dark Pink", "Bright Light Orange")
    )
    for name, cells in regions:
        dress(
            m,
            f"Garden, {name}",
            [c for c in cells if c not in taken],
            ground,
            rng,
            len(cells) // (5 if garden == "wild" else 4),
            flowers,
            **({"wild": True} if garden == "wild" else {}),
        )
    if garden == "wild":
        # A blossom tree in the front corner, leaning over the bed, its crown over the flowers.
        m.use(
            "blossom_tree",
            x=2,
            y=4,
            z=ground,
            lean=2,
            layers=3,
            blossom=("White", "White") if winter else ("Bright Pink", "White"),
            title="Blossom tree",
        )
    return m
