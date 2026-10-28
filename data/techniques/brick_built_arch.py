# SPDX-License-Identifier: Apache-2.0
"""Brick-built arch: a wall with an arched opening through it, the arch brick on the wall to each side and the wall
built over it.

m.use("brick_built_arch", x, y, z, length, rows=None, opening=2, opening_rows=1, offset=None, axis="x",
curved=False, color="Light Bluish Gray", arch_color=None, title="Arch") builds a wall 1 stud thick and `length` studs
long along the axis ("x" or "y") from (x, y), `rows` bricks tall (3 plates each), standing on the studs at height z,
with an opening `opening` studs wide (2 or 4) starting `offset` studs from the wall's start (by default in the
middle; at least 2 studs of wall on each side). The opening's sides rise `opening_rows` rows; then the arch spans it,
its feet on the wall a stud to each side: Arch 1 x 4 (3659, a brick tall) over 2, Arch 1 x 6 x 2 (3307, two bricks)
over 4, or with curved=True two Arch 1 x 3 x 2 with Curved Top (6005) meeting in the middle with a Plate 1 x 2 on
their tabs as a keystone (the curves leave a small open corner over each foot, as they do in real builds). The row over the arch starts with a lintel, a brick 1 x 6 (or 1 x 8) over the arch and a
stud of wall each side, that ties it into the wall; rows (opening_rows + the arch's rows + 1 by default, the
fewest) can add more wall above. The courses are a running bond, every other row starting with a shorter brick. It
brings its own steps, a row at a time, titled "<title>, row 1" and so on, and returns the z of the wall's top.
"""

from emmet_dsl import Brick
from emmet_dsl.core import in_steps

PARTS = [
    "3659",
    "3307",
    "6005",
    "3023b",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "2465",
]


def _pieces(have, run, first=None):
    """A run of `run` studs as lengths, longest first, after an optional shorter first piece."""
    out, left = [], run
    if first and first in have and first <= left:
        out.append(first)
        left -= first
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
    rows=None,
    opening=2,
    opening_rows=1,
    offset=None,
    axis="x",
    curved=False,
    color="Light Bluish Gray",
    arch_color=None,
    title="Arch",
):
    """The wall and its arch; returns the z of the wall's top."""
    if axis not in ("x", "y"):
        raise ValueError(f'brick_built_arch: axis must be "x" or "y", not {axis!r}')
    if opening not in (2, 4) or (curved and opening != 4):
        raise ValueError(
            "brick_built_arch: opening must be 2 or 4 studs (and 4 when curved)"
        )
    tall = 1 if opening == 2 else 2  # the arch's height, in rows
    lintel_row = opening_rows + tall
    rows = lintel_row + 1 if rows is None else rows
    o = (length - opening) // 2 if offset is None else offset
    if opening_rows < 0 or rows < lintel_row + 1:
        raise ValueError(
            f"brick_built_arch: rows must be at least opening_rows + {tall + 1}, here {lintel_row + 1}"
        )
    if o < 2 or length - o - opening < 2:
        raise ValueError(
            "brick_built_arch: the wall needs at least 2 studs each side of the opening"
        )
    arch_color = arch_color or color
    bricks = Brick.lengths(1)
    along = 1 if axis == "x" else 0  # a 1 x n part runs along y at rot=0

    def at(u, h):
        return (x + u, y, h) if axis == "x" else (x, y + u, h)

    def course(r, gaps):
        """The row's wall bricks, around the gaps [(start, end)], as actions."""
        zr = z + 3 * r
        actions, u = [], 0
        for start, end in sorted(gaps) + [(length, length)]:
            run = start - u
            if run > 0:
                # Odd rows start with a shorter brick; a run of 1 or 2 has nothing to stagger.
                first = None
                if r % 2 and run > 2:
                    first = max(n for n in bricks if n <= run) // 2
                sizes = _pieces(bricks, run, first)

                def add(u=u, sizes=sizes, zr=zr):
                    for n in sizes:
                        m.add(bricks[n], color, at=at(u, zr), rot=along)
                        u += n

                actions.append((len(sizes), add))
            u = end
        return actions

    span = (o - 1, o + opening + 1)  # the arch, feet and all
    za = z + 3 * opening_rows
    for r in range(rows):
        zr = z + 3 * r
        if r < opening_rows:
            actions = course(r, [(o, o + opening)])
        elif r < lintel_row:
            actions = course(r, [span])
        elif r == lintel_row:
            lintel = (o - 2, o + opening + 2)
            actions = course(r, [lintel])

            def add_lintel(zr=zr, lintel=lintel):
                m.add(
                    bricks[lintel[1] - lintel[0]],
                    color,
                    at=at(lintel[0], zr),
                    rot=along,
                )

            actions.insert(0, (1, add_lintel))
        else:
            actions = course(r, [])
        if r == opening_rows:
            if curved:
                # rot=0 puts a 6005's foot toward +x; the left half's foot faces the other way.
                turns = (2, 0) if axis == "x" else (3, 1)

                def add_arch(left=turns[0], right=turns[1]):
                    m.add("6005", arch_color, at=at(o - 1, za), rot=left)
                    m.add("6005", arch_color, at=at(o + 2, za), rot=right)

                actions.insert(0, (2, add_arch))
            else:
                part = "3659" if opening == 2 else "3307"

                def add_arch(part=part):
                    m.add(part, arch_color, at=at(o - 1, za), rot=along)

                actions.insert(0, (1, add_arch))
        if curved and r == opening_rows + 1:
            # The keystone on the two tabs, flush with the tops of the curves.
            def add_key():
                m.add("3023b", arch_color, at=at(o + 1, za + 5), rot=along)

            actions.insert(0, (1, add_key))
        in_steps(m, f"{title}, row {r + 1}", actions)
    return z + 3 * rows


def example(m):
    """A wall 10 studs long with an arched opening 4 studs wide through it."""
    with m.step("Footing"):
        m.add("4477", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("brick_built_arch", x=0, y=0, z=1, length=10, opening=4, opening_rows=2)
