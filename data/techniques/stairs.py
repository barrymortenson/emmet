# SPDX-License-Identifier: Apache-2.0
"""Stairs: a brick-built flight of steps, with a tile on each tread, any number of steps.

m.use("stairs", x, y, z, steps, width=2, toward="+x", rise=3, tread=1, color="Light Bluish Gray",
tread_color=None, title="Stairs") builds `steps` steps over the rectangle whose front-left corner is (x, y): `width`
studs across and steps × tread studs along the axis you climb. toward ("+x", "-x", "+y", or "-y") is the way up.
Each step rises `rise` plates (3 is a brick) and is `tread` studs deep. It stands on the studs at height z (a
baseplate, a floor of plates, the top of a wall) and builds as a stepped block, a layer per step, each layer one tread
shorter than the one under it, bricks and then plates, with the strips across the stairs, at widths of 3 or more,
alternating 2 and 1 studs wide from course to course so the layers interlock. A tile on each tread leaves it smooth, with the tread's top at
exactly z + rise × (step number): the first layer is a plate lower to make room for its tile. It brings its own steps,
a layer at a time, titled "<title>, step 1" and so on, and returns the z of the top tread, z + rise × steps.
"""

from emmet_dsl import Brick, Plate, Tile
from emmet_dsl.core import in_steps

PARTS = [
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "3003",
    "3002",
    "3001",
    "2456",
    "3007",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3070b",
    "3069b",
    "63864",
    "2431",
    "6636",
    "3068b",
    "87079",
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


def _strips(width, odd):
    """The widths of the strips across a course: 2s and a 1, or on odd courses a 1 first, so joints alternate."""
    if width <= 2:
        return [width]
    if odd:
        return [1] + [2] * ((width - 1) // 2) + [1] * ((width - 1) % 2)
    return [2] * (width // 2) + [1] * (width % 2)


def build(
    m,
    x,
    y,
    z,
    steps,
    width=2,
    toward="+x",
    rise=3,
    tread=1,
    color="Light Bluish Gray",
    tread_color=None,
    title="Stairs",
):
    """The stairs; returns the z of the top tread."""
    if toward not in ("+x", "-x", "+y", "-y"):
        raise ValueError(
            f'stairs: toward must be "+x", "-x", "+y", or "-y", not {toward!r}'
        )
    if steps < 1 or width < 1 or rise < 1 or tread < 1:
        raise ValueError(
            "stairs needs at least 1 step, and a width, rise, and tread of 1 or more"
        )
    axis, up = toward[1], toward[0] == "+"
    across = "y" if axis == "x" else "x"
    run = steps * tread
    tread_color = tread_color or color

    def corner(u, v):
        """The world (x, y) of the point u studs along the climb and v across, for a box `run` long."""
        return (x + u, y + v) if axis == "x" else (x + v, y + u)

    def span(u0, u1):
        """A span along the climb as the start along the axis, measured from the stairs' low edge."""
        return u0 if up else run - u1

    course = 0
    for k in range(steps):
        u0 = span(k * tread, run)
        length = run - k * tread
        bottom = z if k == 0 else z + rise * k - 1
        height = rise - 1 if k == 0 else rise
        layers = [Brick] * (height // 3) + [Plate] * (height % 3)
        actions = []
        zc = bottom
        for family in layers:
            v = 0
            for w in _strips(width, course % 2):
                u = u0
                pieces = _pieces(family, length, w)

                def strip(pieces=pieces, u=u, v=v, w=w, zc=zc):
                    for n, part in pieces:
                        m.add(
                            part, color, at=(*corner(u, v), zc), rot=_turn(axis, n, w)
                        )
                        u += n

                actions.append((len(pieces), strip))
                v += w
            zc += family.height
            course += 1
        # The tile on this step's tread: strips across the stairs, a tread deep.
        t0 = span(k * tread, (k + 1) * tread)
        top = z + rise * (k + 1) - 1
        for d in _strips(tread, False):
            pieces = _pieces(Tile, width, d)

            def tiles(pieces=pieces, t0=t0, d=d, top=top):
                v = 0
                for n, part in pieces:
                    m.add(
                        part,
                        tread_color,
                        at=(*corner(t0, v), top),
                        rot=_turn(across, n, d),
                    )
                    v += n

            actions.append((len(pieces), tiles))
            t0 += d
        in_steps(m, f"{title}, step {k + 1}", actions)
    return z + rise * steps


def example(m):
    """Four steps, three studs wide, climbing toward +x."""
    m.use("stairs", x=0, y=0, z=0, steps=4, width=3, tread_color="Dark Bluish Gray")
