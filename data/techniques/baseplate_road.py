# SPDX-License-Identifier: Apache-2.0
"""Baseplate road: a smooth road of Dark Bluish Gray tiles with a line of white or yellow tiles down the middle.

m.use("baseplate_road", x, y, z, length, width=9, axis="x", color="Dark Bluish Gray", line_color="White",
dashed=True, dash=3, base=False, base_color=None, title="Road") paves the rectangle whose front-left corner is
(x, y): `length` studs along the axis ("x" or "y") and `width` studs across (3 or more). It stands on the studs at
height z: a baseplate (whose top is z = 0.5 when it sits at 0) or plates. The line is 1 stud wide when the width is
odd and 2 when it's even (two lines side by side, as a double yellow line), with a lane of tiles on each side of it.
dashed=True makes the line dashes `dash` studs long with gaps as long; dashed=False makes it solid. base=True lays a
layer of plates under the road first (in base_color, or the road's color), running across it so the tiles tie them
together, for a road that doesn't sit on a baseplate. It brings its own steps, "<title>, base" when there is one,
then "<title>, far lane", "<title>, line", and "<title>, near lane" (each split into parts when long), and returns the
z of the road's surface.
"""

from emmet_dsl import Plate, Tile
from emmet_dsl.core import in_steps

PARTS = [
    "3070b",
    "3069b",
    "63864",
    "2431",
    "6636",
    "4162",
    "3068b",
    "87079",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "4282",
]


def _pieces(family, run, width):
    """A run of `run` studs, `width` across, as [(length along, part id)], longest first."""
    have = family.lengths(width)
    out, left = [], run
    for n in sorted(have, reverse=True):
        while left >= n:
            out.append((n, have[n]))
            left -= n
    return out


def _turn(axis, along, across):
    """The quarter turn that lays a piece `along` studs along the axis (a piece's long side runs along y at rot=0)."""
    if along == across:
        return 0
    return 1 if (axis == "x") == (along > across) else 0


def _widths(n):
    """Strips 2 studs wide, and one 1 wide when n is odd."""
    return [2] * (n // 2) + [1] * (n % 2)


def build(
    m,
    x,
    y,
    z,
    length,
    width=9,
    axis="x",
    color="Dark Bluish Gray",
    line_color="White",
    dashed=True,
    dash=3,
    base=False,
    base_color=None,
    title="Road",
):
    """The road; returns the z of its surface."""
    if axis not in ("x", "y"):
        raise ValueError(f'baseplate_road: axis must be "x" or "y", not {axis!r}')
    if width < 3 or length < 1 or dash < 1:
        raise ValueError(
            "baseplate_road needs a width of 3 or more, and a length and dash of 1 or more"
        )
    across = "y" if axis == "x" else "x"
    line = 1 if width % 2 else 2
    lane = (width - line) // 2

    def at(u, v, h):
        """The world position u studs along the road and v across it."""
        return (x + u, y + v, h) if axis == "x" else (x + v, y + u, h)

    def strip(family, tint, u, v, run, w, along, h):
        """A strip `run` long and `w` wide, starting at (u, v), laid along the road or across it: (count, add)."""
        pieces = _pieces(family, run, w)
        step = (1, 0) if along == axis else (0, 1)

        def add():
            uu, vv = u, v
            for n, part in pieces:
                m.add(part, tint, at=at(uu, vv, h), rot=_turn(along, n, w))
                uu, vv = uu + n * step[0], vv + n * step[1]

        return (len(pieces), add)

    if base:
        actions, u = [], 0
        for w in _widths(length):
            actions.append(strip(Plate, base_color or color, u, 0, width, w, across, z))
            u += w
        in_steps(m, f"{title}, base", actions)
        z += 1

    def lane_at(v0, widths):
        actions, v = [], v0
        for w in widths:
            actions.append(strip(Tile, color, 0, v, length, w, axis, z))
            v += w
        return actions

    # The wide strips at the road's edges, a narrow one (on an odd lane) by the line.
    in_steps(m, f"{title}, far lane", lane_at(lane + line, _widths(lane)[::-1]))
    actions, u, on = [], 0, True
    while u < length:
        n = min(dash, length - u) if dashed else length
        tint = line_color if on or not dashed else color
        actions.append(strip(Tile, tint, u, lane, n, line, axis, z))
        u += n
        on = not on
    in_steps(m, f"{title}, line", actions)
    in_steps(m, f"{title}, near lane", lane_at(0, _widths(lane)))
    return z + 1


def example(m):
    """A road across a 16 x 16 baseplate, with a dashed white line."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("baseplate_road", x=0, y=3, z=0.5, length=16, width=9, axis="x")
