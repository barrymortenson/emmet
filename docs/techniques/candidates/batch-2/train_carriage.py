# SPDX-License-Identifier: Apache-2.0
"""Train carriage: a 6-wide passenger carriage's body on a train base, window bands along both sides, doors at the ends,
and a roof that curves down at its edges.

m.use("train_carriage", x, y, z, length=16, color="Red", window="White", roof="Light Bluish Gray", title="Carriage")
builds on a train base (6584) or a plate whose 6 x `length` top is at (x, y, z) (the base runs along x): a course of
bricks along each side, a band of windows 1 x 2 x 2 with glass along each side between 1-wide posts, a course of plates
over them, and a roof: plates across the full width, which tie the two sides together, and on them curved slopes 3 x 2
falling to each edge, two meeting in the middle, so the roof is round. The ends are left open for the next car's
gangway. Lengths are multiples of 4 from 8 to 24. Returns the z of the roof's top.
"""

from emmet_dsl import Brick, Plate
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "6584",
    "3028",
    "60592",
    "60601",
    "3005",
    "3004",
    "3010",
    "3008",
    "3009",
    "3460",
    "3666",
    "3710",
    "24309",
    "3795",
    "3622",
]


def _window(m, color, at):
    f = m.add("60592", color, at=at, rot=1)
    (x0, y0, z0), (x1, y1, _) = f.min, f.max
    m.add(
        "60601",
        "Trans-Clear",
        at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
        rot=1,
    )


def build(
    m,
    x,
    y,
    z,
    length=16,
    color="Red",
    window="White",
    roof="Light Bluish Gray",
    title="Carriage",
):
    """The sides, windows, and roof; returns the z of the roof's top."""
    if length % 4 or not 8 <= length <= 24:
        raise ValueError(
            f"train_carriage takes a length of 8 to 24 in steps of 4, not {length}"
        )
    sides = (y, y + 5)
    with m.step(f"{title}: lower sides"):
        for sy in sides:
            m.row(Brick, color, at=(x, sy, z), length=length)
    for sy in sides:
        # A post at each end and between pairs of windows: post, window, window, post... A window and its glass
        # stay in one step.
        actions, a = [], x
        while a < x + length:
            actions.append(
                (
                    2,
                    lambda a=a, sy=sy: [
                        m.add("3005", color, at=(a, sy, z + 3 + 3 * r))
                        for r in range(2)
                    ],
                )
            )
            a += 1
            if a + 2 <= x + length - 1:
                actions.append(
                    (2, lambda a=a, sy=sy: _window(m, window, (a, sy, z + 3)))
                )
                a += 2
        in_steps(m, f"{title}: windows, {'left' if sy == y else 'right'} side", actions)
    with m.step(f"{title}: top plates"):
        for sy in sides:
            m.row(Plate, color, at=(x, sy, z + 9), length=length)
    # Plates 2 x 6 across the carriage, each on both sides' walls: they tie the sides together.
    in_steps(
        m,
        f"{title}: roof plates",
        [
            (1, lambda a=a: m.add("3795", roof, at=(x + a, y, z + 10), rot=0))
            for a in range(0, length, 2)
        ],
    )
    for half, (sy, turn) in (("left", (y, 3)), ("right", (y + 3, 1))):
        # Curved 3 x 2 slopes, 3 long across the roof, falling to its edge: two meet in the middle.
        with m.step(f"{title}: roof, {half} half"):
            for a in range(0, length, 2):
                m.add("24309", roof, at=(x + a, sy, z + 11), rot=turn)
    return z + 14


def example(m):
    """A carriage 12 long, on a 6 x 12 plate standing in for its train base."""
    with m.step("Base"):
        m.add("3028", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("train_carriage", x=0, y=0, z=1, length=12)
