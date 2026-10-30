# SPDX-License-Identifier: Apache-2.0
"""Architecture base: the finished display base of an architecture model, a dark stepped frame round a tiled ground
strip, with a long name tile set in the front edge and bare studs only where the model and its trees stand.

m.use("architecture_base", x, y, z, length=12, depth=10, site=(2, 2, 4, 4), studs=(), ground="Light Bluish Gray",
frame="Black", step="Dark Bluish Gray", name="White", name_length=6, title="Base") builds, from z, a base
`length` x `depth` studs from (x, y): a layer of plates in `frame`; on it a 1-wide rim of tiles in `frame` with the name
tile (a 1 x `name_length` tile in `name`) in the middle of the front rim, and plates in `step` inside it; on those a
second rim of tiles in `step`, set in a stud, so the frame steps down twice; and the ground inside, tiled in `ground`
except the site and the single studs in `studs`, left bare. site: (x, y, length, depth) and studs: (x, y) cells, both
counted from the ground's own corner, (x + 2, y + 2). Returns the z of the ground's top, where the model stands on the
site's studs.
"""

from emmet_dsl import Plate, Tile
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21061, set 21066, and set
21067. Not derived from any model file."""

PARTS = [
    "3034",
    "3832",
    "3795",
    "3666",
    "3460",
    "6636",
    "4162",
    "2431",
    "3069b",
    "3070b",
    "87079",
    "3068b",
]

TILES = [
    ((2, 4), "87079"),
    ((2, 2), "3068b"),
    ((1, 4), "2431"),
    ((1, 2), "3069b"),
    ((1, 1), "3070b"),
]
NAME = {4: "2431", 6: "6636", 8: "4162"}


def _ring(m, color, x, y, z, w, d, gap=None):
    """Tile actions for a 1-wide ring round a w x d rectangle; gap: (x0, n) a run of the front row left out."""
    acts = []

    def run(at, n, axis):
        for h in _pieces(n):
            a = at
            acts.append(
                (1, lambda a=a, h=h: m.row(Tile, color, at=a, length=h, axis=axis))
            )
            at = (at[0] + h, at[1], at[2]) if axis == "x" else (at[0], at[1] + h, at[2])

    if gap:
        g0, gn = gap
        run((x, y, z), g0 - x, "x")
        run((g0 + gn, y, z), x + w - g0 - gn, "x")
    else:
        run((x, y, z), w, "x")
    run((x, y + d - 1, z), w, "x")
    run((x, y + 1, z), d - 2, "y")
    run((x + w - 1, y + 1, z), d - 2, "y")
    return acts


def _pieces(n):
    out = []
    for size in (8, 6, 4, 3, 2, 1):
        while n >= size:
            out.append(size)
            n -= size
    return out


def build(
    m,
    x,
    y,
    z,
    length=12,
    depth=10,
    site=(2, 2, 4, 4),
    studs=(),
    ground="Light Bluish Gray",
    frame="Black",
    step="Dark Bluish Gray",
    name="White",
    name_length=6,
    title="Base",
):
    """The plates, the two rims, the name tile, and the ground; returns the z of the ground's top."""
    gw, gd = length - 4, depth - 4
    if gw < 2 or gd < 2 or name_length not in NAME or name_length > length - 2:
        raise ValueError(
            f"architecture_base takes a length and depth of 6 or more and a name_length of 4, 6 or 8 that fits the "
            f"front, not {length}, {depth}, {name_length}"
        )
    sx, sy, sw, sd = site
    bare = {(sx + i, sy + j) for i in range(sw) for j in range(sd)} | {
        tuple(s) for s in studs
    }
    if any(not (0 <= i < gw and 0 <= j < gd) for i, j in bare):
        raise ValueError(
            f"architecture_base: the site and studs must lie in the ground's {gw} x {gd} studs"
        )
    with m.step(f"{title}, plates"):
        m.fill(Plate, frame, at=(x, y, z), size=(length, depth))
    with m.step(f"{title}, inner plates"):
        m.fill(Plate, step, at=(x + 1, y + 1, z + 1), size=(length - 2, depth - 2))
    g0 = x + (length - name_length) // 2
    acts = _ring(m, frame, x, y, z + 1, length, depth, gap=(g0, name_length))
    acts.insert(
        0, (1, lambda: m.add(NAME[name_length], name, at=(g0, y, z + 1), rot=1))
    )
    in_steps(m, f"{title}, frame", acts)
    in_steps(
        m, f"{title}, step", _ring(m, step, x + 1, y + 1, z + 2, length - 2, depth - 2)
    )
    covered, acts = set(bare), []
    for j in range(gd):
        for i in range(gw):
            if (i, j) in covered:
                continue
            for (w, d), part in TILES:
                for tw, td, t in ((d, w, 1), (w, d, 0)):
                    cells = {(i + a, j + b) for a in range(tw) for b in range(td)}
                    if i + tw <= gw and j + td <= gd and not cells & covered:
                        covered |= cells
                        at = (x + 2 + i, y + 2 + j, z + 2)
                        acts.append(
                            (
                                1,
                                lambda p=part, at=at, t=t: m.add(
                                    p, ground, at=at, rot=t
                                ),
                            )
                        )
                        break
                else:
                    continue
                break
    in_steps(m, f"{title}, ground", acts)
    return z + 2


def example(m):
    """A base 12 x 10 with a small tan building on its site and a tree beside it."""
    top = m.use("architecture_base", x=0, y=0, z=0, site=(1, 1, 4, 4), studs=[(6, 4)])
    with m.step("Building"):
        m.wall(3, 3, top, 4, depth=4, color="Tan", rows=2)
    with m.step("Building, roof"):
        m.add("3031", "Dark Tan", at=(3, 3, top + 6))
    with m.step("Tree"):
        m.add("3062b", "Reddish Brown", at=(8, 6, top))
        m.add("3062b", "Green", at=(8, 6, top + 3))
