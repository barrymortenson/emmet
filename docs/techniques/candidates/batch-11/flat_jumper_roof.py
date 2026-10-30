# SPDX-License-Identifier: Apache-2.0
"""Flat jumper roof: a flat roof that lifts off, tiled smooth in the middle and ringed with jumper plates so its edge
reads as a row of blocks, one stud each.

m.use("flat_jumper_roof", x, y, z, length, depth, base="Dark Bluish Gray", tiles="Dark Bluish Gray",
rim="Light Bluish Gray", title="Roof") builds a length x depth roof (even, 4 or more) apart, as a submodel, and seats
it on the walls whose outside is length x depth from (x, y), on their top at z. The roof is a layer of plates in the
base color, then on it a 1-stud ring of 1 x 2 jumpers in the rim color around the edge (along the front and back, and
up the sides between them) and the middle covered with 2 x 4 and 2 x 2 tiles in the tiles color, their seams a stud
off the plates' so the two layers lock. Only the jumpers' studs show, one a 2-stud block. Lift it off to reach inside.
It's a submodel, "<title>, plates", "<title>, rim", and "<title>, tiles", placed in a step titled "<title>"; it
returns the placed copy's Handle, whose .max[2] is the roof's top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3034", "3832", "3460", "3795", "3020", "3022", "3794b", "87079", "3068b"]


def _roof(m, length, depth, base, tiles, rim, title):
    """The roof in its own frame, its corner at the origin."""
    with m.step(f"{title}, plates"):
        m.fill(Plate, base, at=(0, 0, 0), size=(length, depth))
    ring = [(i, 0, 1) for i in range(0, length, 2)] + [
        (i, depth - 1, 1) for i in range(0, length, 2)
    ]
    ring += [(0, j, 0) for j in range(1, depth - 1, 2)] + [
        (length - 1, j, 0) for j in range(1, depth - 1, 2)
    ]
    for s in range(0, len(ring), 8):
        with m.step(f"{title}, rim"):
            for i, j, turn in ring[s : s + 8]:
                m.add("3794b", rim, at=(i, j, 1), rot=turn)
    # The middle in 2-wide strips along the long side, 2 x 4 tiles and a 2 x 2 to finish a strip.
    inner = (length - 2, depth - 2)
    along_x = inner[0] >= inner[1]
    long, short = (inner[0], inner[1]) if along_x else (inner[1], inner[0])
    cells = []
    for b in range(0, short, 2):
        a = 0
        while a < long:
            n = 4 if long - a >= 4 else 2
            cells.append((a, b, n))
            a += n
    for s in range(0, len(cells), 6):
        with m.step(f"{title}, tiles"):
            for a, b, n in cells[s : s + 6]:
                part = "87079" if n == 4 else "3068b"
                if along_x:
                    m.add(part, tiles, at=(1 + a, 1 + b, 1), rot=1)
                else:
                    m.add(part, tiles, at=(1 + b, 1 + a, 1))


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    base="Dark Bluish Gray",
    tiles="Dark Bluish Gray",
    rim="Light Bluish Gray",
    title="Roof",
):
    """The roof, seated on the walls; returns the placed copy's Handle."""
    if length % 2 or depth % 2 or length < 4 or depth < 4:
        raise ValueError(
            f"flat_jumper_roof takes an even length and depth of 4 or more, not {length}, {depth}"
        )
    roof = m.submodel(
        "flat_jumper_roof",
        _roof,
        length=length,
        depth=depth,
        base=base,
        tiles=tiles,
        rim=rim,
        title=title,
    )
    with m.step(title):
        return m.place(roof, at=(x, y, z))


def example(m):
    """A flat roof on a shed 6 x 10 studs, two bricks tall."""
    with m.step("Floor"):
        m.add("3036", "Dark Bluish Gray", at=(0, 0, 0))
        m.add("3795", "Dark Bluish Gray", at=(0, 8, 0), rot=1)
    for r in range(2):
        with m.step(f"Walls, row {r + 1}"):
            m.wall(0, 0, 1 + 3 * r, 6, 10, color="Light Bluish Gray")
    m.use("flat_jumper_roof", x=0, y=0, z=7, length=6, depth=10)
