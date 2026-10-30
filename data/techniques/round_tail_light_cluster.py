# SPDX-License-Identifier: Apache-2.0
"""Round tail light cluster: a classic car's small round tail lights in pairs at each corner of the rear panel, on
bricks with a stud on the side, over a silver bumper of tiles on side studs.

m.use("round_tail_light_cluster", x, y, z, width=8, lights=2, color="Dark Green", lamp="Trans Red", bumper="Pearl Light
Gray", title="Tail lights") lays two courses across a car's rear at x, `width` studs along y from (y, z), facing +x: a
course of bricks 1 x 4 with studs on the side (30414) with tiles 1 x n in `bumper` (2431, 3069b) along their studs, the
bumper; then a course of bricks with, at each end, `lights` bricks 1 x 1 with a stud on the side (87087) carrying round
tiles 1 x 1 (98138) in `lamp`, and bricks between. width is a multiple of 4, 8 to 16; lights 1 or 2. Returns the z of
the second course's top (z + 6).
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11381. Not derived from any
model file."""

PARTS = [
    "30414",
    "2431",
    "3069b",
    "87087",
    "98138",
    "3004",
    "3010",
    "3622",
    "3009",
    "3008",
]

LAMP = Rot(y=90)  # a tile on its back, its face to +x
STRIP = Rot(y=270, z=180)  # a tile 1 x n on its back, its face to +x, long along y


def build(
    m,
    x,
    y,
    z,
    width=8,
    lights=2,
    color="Dark Green",
    lamp="Trans Red",
    bumper="Pearl Light Gray",
    title="Tail lights",
):
    """The bumper course and the lights course; returns the z of the top."""
    if width % 4 or not 8 <= width <= 16:
        raise ValueError(
            f"round_tail_light_cluster takes a width of 8, 12, or 16, not {width}"
        )
    if lights not in (1, 2):
        raise ValueError(
            f"round_tail_light_cluster takes 1 or 2 lights a side, not {lights}"
        )
    face = x + 1  # the courses' rear face; the side studs stick out of it
    with m.step(f"{title}: bumper course"):
        for k in range(0, width, 4):
            m.add("30414", color, at=(x, y + k, z), rot=0)  # its side studs to the rear
    # A side stud is 1.75 plates up its brick; a tile on it is 2.5 plates tall, centered on it.
    with m.step(f"{title}: bumper"):
        for k in range(0, width, 4):
            m.add("2431", bumper, at=(face, y + k, z + 0.5), rot=STRIP)
    zl = z + 3
    sides = [*range(lights), *range(width - lights, width)]
    with m.step(f"{title}: light course"):
        for k in sides:
            m.add("87087", color, at=(x, y + k, zl), rot=0)
        m.row(Brick, color, at=(x, y + lights, zl), length=width - 2 * lights, axis="y")
    with m.step(f"{title}: lamps"):
        for k in sides:
            m.add("98138", lamp, at=(face, y + k, zl + 0.5), rot=LAMP)
    return z + 6


def example(m):
    """The rear of a car 8 studs wide."""
    with m.step("Floor"):
        m.add("3035", "Black", at=(0, 0, 0), rot=0)
    with m.step("Body"):
        m.add("3008", "Dark Green", at=(0, 0, 1), rot=0)
    m.use("round_tail_light_cluster", x=3, y=0, z=1)
