# SPDX-License-Identifier: Apache-2.0
"""Fence: a straight fence of 1 x 4 fence panels and round posts, of any length (4 or more), along x or y.

m.use("fence", x, y, z, length, axis="x", kind="lattice", color="White", base=True, base_color=None,
title="Fence") runs `length` studs along the axis from (x, y), one stud deep. kind picks the panel: "lattice"
(Fence Lattice 1 x 4 x 2, 3185) or "spindled" (Fence Spindled 1 x 4 x 2, 30055), both 6 plates tall, or "low"
(Fence Lattice 1 x 4 x 1, 3633), 3 plates tall. Panels are 4 studs long; a length that isn't a multiple of 4 gets
posts of 1 x 1 round bricks (3062b) stacked to the fence's height: one at the start for 1 stud left over, one at each
end for 2, and one in the middle as well for 3. It stands on the studs at height z. With base=True (the default) a
row of 1-wide plates goes under it first, with no joint under a joint between panels and posts, so the plates tie the
fence into one piece (a fence on a baseplate can do without: base=False). It brings its own steps, "<title>, base"
and "<title>, panels" (split into parts when long), and returns the z of the fence's top.
"""

from emmet_dsl import Plate
from emmet_dsl.core import in_steps

PANELS = {"lattice": ("3185", 6), "spindled": ("30055", 6), "low": ("3633", 3)}
POST = "3062b"

PARTS = [
    "3185",
    "30055",
    "3633",
    "3062b",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
]


def _layout(length):
    """The fence's pieces in order: ("post", 1) and ("panel", 4)."""
    left = length % 4
    panels = [("panel", 4)] * (length // 4)
    middle = len(panels) // 2
    if left == 3:
        panels.insert(middle, ("post", 1))
    if left >= 1:
        panels.insert(0, ("post", 1))
    if left >= 2:
        panels.append(("post", 1))
    return panels


def _base_run(length, joints, have):
    """Plate lengths that cover `length` with no joint at any of `joints` (the fewest pieces), or longest first when
    that can't be done."""
    best = {0: []}
    for pos in range(1, length + 1):
        if pos in joints and pos != length:
            continue
        options = [best[pos - n] + [n] for n in have if pos - n in best and n <= pos]
        if options:
            best[pos] = min(options, key=len)
    if length in best:
        return best[length]
    out, left = [], length
    for n in sorted(have, reverse=True):
        while left >= n:
            out.append(n)
            left -= n
    return out


def build(
    m,
    x,
    y,
    z,
    length,
    axis="x",
    kind="lattice",
    color="White",
    base=True,
    base_color=None,
    title="Fence",
):
    """The fence; returns the z of its top."""
    if kind not in PANELS:
        raise ValueError(
            f"fence: kind must be one of {', '.join(PANELS)}, not {kind!r}"
        )
    if axis not in ("x", "y"):
        raise ValueError(f'fence: axis must be "x" or "y", not {axis!r}')
    if length < 4:
        raise ValueError(f"fence needs a length of 4 or more, not {length}")
    panel, height = PANELS[kind]
    turn = 1 if axis == "x" else 0  # a 1 x n part runs along y at rot=0

    def at(u, h):
        return (x + u, y, h) if axis == "x" else (x, y + u, h)

    layout = _layout(length)
    joints, u = set(), 0
    for _, n in layout:
        u += n
        joints.add(u)
    if base:
        plates = Plate.lengths(1)
        run = _base_run(length, joints, plates)
        actions = []
        u = 0
        for n in run:

            def plate(n=n, u=u):
                m.add(plates[n], base_color or color, at=at(u, z), rot=turn)

            actions.append((1, plate))
            u += n
        in_steps(m, f"{title}, base", actions)
        z += 1
    actions, u = [], 0
    for what, n in layout:
        if what == "panel":

            def add(u=u):
                m.add(panel, color, at=at(u, z), rot=turn)

            actions.append((1, add))
        else:

            def add(u=u):
                m.stack(POST, color, at=at(u, z), count=height // 3)

            actions.append((height // 3, add))
        u += n
    in_steps(m, f"{title}, panels", actions)
    return z + height


def example(m):
    """An 11-stud lattice fence along x: a post at each end and in the middle, on a base of plates."""
    m.use("fence", x=0, y=0, z=0, length=11, axis="x", kind="lattice", color="White")
