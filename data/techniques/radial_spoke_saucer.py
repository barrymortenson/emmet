# SPDX-License-Identifier: Apache-2.0
"""Radial spoke saucer: the frame of a big flat disc that stays rigid without a baseplate, eight spliced spokes of
plates from a hub, with a lit window block at the tip of each straight spoke.

m.use("radial_spoke_saucer", x, y, z, reach=6, color="Tan", rim="White", glass="Trans Light Blue", title="Spokes")
builds, from z, a hub 4 x 4 studs centered on (x, y) (a 4 x 4 plate, with a 2 x 2 round brick and a dish on it as the
dome) and eight spokes two plates thick. The four straight ones run `reach` studs out from the hub along +x, -x, +y and
-y, 2 studs wide: a plate from the hub's edge under a plate that starts a stud inside the hub, so the joints of the two
layers miss each other and every spoke is a spliced beam. The four diagonals are stepped chains of 2 x 2 plates from the
hub's corners, the upper layer over the corners of the lower one. At each straight spoke's tip a 1 x 2 brick with studs
on its side faces out in `rim`, with a 1 x 2 tile in `glass` on it as the window band. Petals or plates laid over it
later lock the spokes together. reach: 4 to 10; the diagonals reach about as far. Returns the z of the spokes' top.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10356. Not derived from any
model file."""

PARTS = ["3031", "3022", "3795", "3034", "3832", "11211", "3069b", "3941", "4740"]

# By the way a straight spoke runs: its unit vector, the quarter turn that points a side-stud brick out, and the
# turn of a tile on its back facing out.
SPOKES = [
    ((1, 0), 0, Rot(y=-90, z=180)),
    ((0, 1), 1, Rot(y=-90, z=270)),
    ((-1, 0), 2, Rot(y=-90)),
    ((0, -1), 3, Rot(y=-90, z=90)),
]
OUT = 0.4  # a tile on its back stands out this far, in studs


def _box(cx, cy, d, a, b, w=2):
    """The min corner of the w-wide strip of a spoke running along d, from a to b studs out from (cx, cy)."""
    dx, dy = d
    if dx:
        x0 = cx + a if dx > 0 else cx - b
        return x0, cy - w / 2
    y0 = cy + a if dy > 0 else cy - b
    return cx - w / 2, y0


def build(
    m,
    x,
    y,
    z,
    reach=6,
    color="Tan",
    rim="White",
    glass="Trans Light Blue",
    title="Spokes",
):
    """The hub, the eight spokes, and the window blocks; returns the z of the spokes' top."""
    if not 4 <= reach <= 10:
        raise ValueError(f"radial_spoke_saucer takes a reach of 4 to 10, not {reach}")
    with m.step(f"{title}, hub"):
        m.add("3031", color, at=(x - 2, y - 2, z))
    for i, (d, turn, face) in enumerate(SPOKES):
        along = d[0] != 0
        with m.step(f"{title}, spoke {i + 1}"):
            # The lower plate from the hub's edge, the upper from a stud inside it, a stud short of the tip.
            m.add(
                Plate(2, reach),
                color,
                at=(*_box(x, y, d, 2, 2 + reach), z),
                rot=1 if along else 0,
            )
            m.add(
                Plate(2, reach),
                color,
                at=(*_box(x, y, d, 1, 1 + reach), z + 1),
                rot=1 if along else 0,
            )
            bx, by = _box(x, y, d, 1 + reach, 2 + reach)
            m.add("11211", rim, at=(bx, by, z + 1), rot=turn)
            tx, ty = (
                bx + (1 if d == (1, 0) else -OUT if d == (-1, 0) else 0),
                by + (1 if d == (0, 1) else -OUT if d == (0, -1) else 0),
            )
            m.add("3069b", glass, at=(tx, ty, z + 1.5), rot=face)
    n = max(
        1, round(((reach + 2) / 1.41 - 2) / 2)
    )  # a diagonal ends about as far out as a straight spoke
    for i, (sx, sy) in enumerate(((1, 1), (-1, 1), (-1, -1), (1, -1))):
        with m.step(f"{title}, diagonal {i + 1}"):
            for k in range(n):
                for layer, start in ((0, 2), (1, 1)):
                    lo = start + 2 * k
                    px = x + lo if sx > 0 else x - lo - 2
                    py = y + lo if sy > 0 else y - lo - 2
                    m.add("3022", color, at=(px, py, z + layer))
    with m.step(f"{title}, dome"):
        m.add("3941", color, at=(x - 1, y - 1, z + 1))
        m.add("4740", rim, at=(x - 1, y - 1, z + 4))
    return z + 2


def example(m):
    """A saucer frame with spokes 6 studs long, on a pylon."""
    with m.step("Pylon"):
        m.add("3031", "Black", at=(8, 8, 0))
        m.stack("3941", "Dark Bluish Gray", at=(9, 9, 1), count=2)
    m.use("radial_spoke_saucer", x=10, y=10, z=7)
