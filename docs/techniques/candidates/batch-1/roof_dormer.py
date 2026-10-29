# SPDX-License-Identifier: Apache-2.0
"""Roof dormer: a small window that stands up out of a sloped roof, with its own little gable roof.

m.use("roof_dormer", x, y, z, color="White", roof="Dark Red", title="Dormer") builds, in the bottom course of a roof
of 45° slopes facing front (-y) whose front edge is at y and which starts at z, a dormer 4 studs wide from x: a window
1 x 2 x 2 with its glass between two cheeks of 1 x 1 bricks, a 1 x 4 brick behind it carrying the next course, and two
2 x 2 slopes facing left and right over it as its gable. Lay the bottom course's slopes around it (leaving x to x + 4),
and the next course a stud back as usual, then call the function it returns, which puts the gable on (the next course
has to go in first: the gable closes the way in to it). Returns that function.
"""

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 21335, set 43217, set 10350. Not derived
from any model file."""

PARTS = [
    "60592",
    "60601",
    "3005",
    "3010",
    "3039",
    "3037",
    "3040b",
    "3009",
    "3958",
    "3004",
]


def build(m, x, y, z, color="White", roof="Dark Red", title="Dormer"):
    """The dormer's cheeks, window, and back brick; returns a function that adds its gable."""
    with m.step(f"{title}: window"):
        for r in range(2):
            m.add("3005", color, at=(x, y, z + 3 * r))
            m.add("3005", color, at=(x + 3, y, z + 3 * r))
        f = m.add("60592", color, at=(x + 1, y, z), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601",
            "Trans-Clear",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
        )
        m.add("3010", roof, at=(x, y + 1, z), rot=1)

    def gable():
        with m.step(f"{title}: gable"):
            m.add("3039", roof, at=(x, y, z + 6), rot=2)
            m.add("3039", roof, at=(x + 2, y, z + 6), rot=0)

    return gable


def example(m):
    """A short roof, two courses deep, with a dormer in the middle of the bottom course."""
    with m.step("Base"):
        m.add("3958", "Light Bluish Gray", at=(0, 0, 0))
    with m.step("Roof, course 1"):
        m.add("3040b", "Dark Red", at=(0, 0, 1), rot=3)
        m.add("3040b", "Dark Red", at=(5, 0, 1), rot=3)
        m.add("3009", "Dark Red", at=(0, 2, 1), rot=1)
    gable = m.use("roof_dormer", x=1, y=0, z=1)
    with m.step("Roof, course 2"):
        m.add("3037", "Dark Red", at=(0, 1, 4), rot=3)
        m.add("3039", "Dark Red", at=(4, 1, 4), rot=3)
    gable()
