# SPDX-License-Identifier: Apache-2.0
"""Round bastion: a short round pier with a smooth face, for the rounded end of a wall run or a slim tower, 2 studs
across.

m.use("round_bastion", x, y, z, rows=4, wall=4, dome=True, color="White", title="Bastion") stacks, on the studs at z, a
round pier 2 x 2 from (x, y): `rows` round bricks 2 x 2 (6143), a round plate 2 x 2 between every second pair so the
face is banded, and on top a dome (Dish 2 x 2 Inverted, 4740), or with dome=False a round tile 2 x 2. With wall, a wall
run a stud thick comes in from the left along y's front row, `wall` studs long (x - wall to x), `rows` - 1 courses of
bricks in running bond, capped with bricks 1 x 2 with a barrel top (37352) and, against the pier, a brick 2 x 1 x 1 1/3
whose curved top (6091) rises to meet it, so the pier reads as the wall's rounded end. The wall goes up with the pier, a
course at a time. rows: 2 to 10; wall: 0 or 2 to 12. Returns the z of the pier's top.
"""

from emmet_dsl import Brick

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11377. Not derived from any
model file."""

PARTS = [
    "6143",
    "4032b",
    "4740",
    "4150",
    "37352",
    "6091",
    "3004",
    "3005",
    "3622",
    "3010",
    "3009",
    "3008",
]


def build(m, x, y, z, rows=4, wall=4, dome=True, color="White", title="Bastion"):
    """The pier and its wall, a course at a time, and the caps; returns the z of the pier's top."""
    if not 2 <= rows <= 10 or wall == 1 or not 0 <= wall <= 12:
        raise ValueError(
            f"round_bastion takes 2 to 10 rows and a wall of 0 or 2 to 12, not {rows}, {wall}"
        )
    zp = z
    for r in range(rows):
        with m.step(f"{title}, course {r + 1}"):
            m.add("6143", color, at=(x, y, zp))
            if wall and r < rows - 1:
                m.row(
                    Brick,
                    color,
                    at=(x - wall, y, z + 3 * r),
                    length=wall,
                    first=1 if r % 2 and wall > 2 else None,
                )
            elif wall:
                # The cap: barrel-topped bricks, and against the pier a curved top that rises to meet it.
                if wall % 2:
                    m.add("3005", color, at=(x - wall, y, z + 3 * r))
                for k in range(wall % 2, wall - 2, 2):
                    m.add("37352", color, at=(x - wall + k, y, z + 3 * r), rot=1)
                m.add("6091", color, at=(x - 2, y, z + 3 * r), rot=0)
        zp += 3
        if r % 2 and r < rows - 1:
            with m.step(f"{title}, band over course {r + 1}"):
                m.add("4032b", color, at=(x, y, zp))
            zp += 1
    with m.step(f"{title}, top"):
        m.add("4740" if dome else "4150", color, at=(x, y, zp))
    return zp + 1


def example(m):
    """A bastion at the end of a short wall, on a strip of ground."""
    from emmet_dsl import Plate

    with m.step("Ground"):
        m.row(Plate, "Dark Bluish Gray", at=(-4, 0, 0), length=6, width=2)
    m.use("round_bastion", x=0, y=0, z=1)
