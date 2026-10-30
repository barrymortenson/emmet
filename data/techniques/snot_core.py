# SPDX-License-Identifier: Apache-2.0
"""SNOT core: the hidden block in the middle of a sculpture, with studs pointing out on all four sides as well as up, so
the skin can be pressed on every face.

m.use("snot_core", x, y, z, size=2, courses=2, color="Red", title="Core") builds a column `size` x `size` studs (1 to 4)
from (x, y), on the studs at height z, `courses` courses tall. Each course is two layers of plates, their seams crossed,
that tie the course below, then a layer of bricks with studs on the side: at each corner a brick 1 x 1 with studs on two
adjacent sides (a brick with studs on four sides for a 1 x 1 core) turned so its studs point out of both faces, along
each edge a brick 1 x 1 with a stud on one side pointing out, and plain bricks inside. A course is 5 plates (2 studs)
tall, and its side studs are 3.75 plates above its foot, so a tile or plate 2 studs tall hung on a course's studs by its
upper row sits flush with the course, top and bottom, and courses of panels stack without gaps. The color is the
builder's: it never shows once the skin is on, and a bright one tells the core from the skin. It brings its own steps, a
course at a time, and returns the z of its top, where a cap or the next part goes.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21350, 43014, 21368, and
43290. Not derived from any model file."""

PARTS = [
    "26604",
    "87087",
    "4733",
    "3005",
    "3024",
    "3023b",
    "3623",
    "3022",
    "3021",
    "3031",
    "3020",
]

# Where a cell's side studs point, by quarter turn: a 1 x 1 brick with a stud on one side points it +x at rot 0, +y at
# rot 1, -x at rot 2, -y at rot 3; one with studs on two adjacent sides points them at rot and rot + 1.
CORNERS = {
    (0, 0): 2,
    (1, 0): 3,
    (1, 1): 0,
    (0, 1): 1,
}  # front-left, front-right, back-right, back-left
EDGES = {"front": 3, "back": 1, "left": 2, "right": 0}


def _turn(i, j, size):
    """The brick and its quarter turn for the cell (i, j) of a size x size course; None for a plain inside cell."""
    if size == 1:
        return "4733", 0
    hi = size - 1
    if i in (0, hi) and j in (0, hi):
        return "26604", CORNERS[(i // hi, j // hi)]
    if j == 0:
        return "87087", EDGES["front"]
    if j == hi:
        return "87087", EDGES["back"]
    if i == 0:
        return "87087", EDGES["left"]
    if i == hi:
        return "87087", EDGES["right"]
    return None


def build(m, x, y, z, size=2, courses=2, color="Red", title="Core"):
    """The courses of plates and studded bricks; returns the z of the top."""
    if not 1 <= size <= 4 or courses < 1:
        raise ValueError(
            f"snot_core takes a size of 1 to 4 and 1 or more courses, not {size}, {courses}"
        )
    for c in range(courses):
        zc = z + 5 * c
        with m.step(f"{title}, course {c + 1}: plates"):
            m.fill(Plate, color, at=(x, y, zc), size=(size, size))
            if size == 3:
                # A 3 x 3 fills as a 2 x 3 and a 1 x 3 one way: the second layer runs the other way, across them.
                for i in range(3):
                    m.add(Plate(1, 3), color, at=(x + i, y, zc + 1))
            elif size > 1:
                m.fill(Plate, color, at=(x, y, zc + 1), size=(size, size))
            else:
                m.add("3024", color, at=(x, y, zc + 1))
        per = max(1, 8 // size)  # rows of bricks a step, front first
        for j0 in range(0, size, per):
            part_of = f" ({j0 // per + 1} of {-(-size // per)})" if size > per else ""
            with m.step(f"{title}, course {c + 1}: bricks{part_of}"):
                for j in range(j0, min(size, j0 + per)):
                    for i in range(size):
                        turn = _turn(i, j, size)
                        part, rot = turn if turn else ("3005", 0)
                        m.add(part, color, at=(x + i, y + j, zc + 2), rot=rot)
    return z + 5 * courses


def example(m):
    """A 2 x 2 core three courses tall, with white tiles pressed on the front and left of its lower two courses."""
    from emmet_dsl import Rot

    with m.step("Base"):
        m.add("3031", "Light Bluish Gray", at=(-1, -1, 0))
    top = m.use("snot_core", x=0, y=0, z=1, size=2, courses=3)
    for c in range(2):
        zc = 1 + 5 * c
        with m.step(f"Skin, course {c + 1}"):
            m.add(
                "3068b", "White", at=(0, -0.4, zc), rot=Rot(y=-90, z=90)
            )  # on the front studs
            m.add(
                "3068b", "White", at=(-0.4, 0, zc), rot=Rot(y=-90)
            )  # on the left studs
    with m.step("Cap"):
        m.add("3068b", "White", at=(0, 0, top))
