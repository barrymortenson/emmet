# SPDX-License-Identifier: Apache-2.0
"""Curved dormer: a small dormer with a rounded roof of curved slopes, dark cheeks either side of its window, set
in the bottom course of a roof of 45° slopes.

m.use("curved_dormer", x, y, z, color="White", cheeks="Black", roof="White", title="Dormer") builds, in the bottom
course of a roof of 45° slopes facing front (-y) whose front edge is at y and which starts at z, a dormer 4 studs
wide from x: a window 1 x 2 x 2 with its glass between two cheeks of `cheeks` 1 x 1 bricks, and a 1 x 4 brick behind
it carrying the next course. Lay the bottom course's slopes around it (leaving x to x + 4), and the next course a
stud back as usual, then call the function it returns, which puts the roof on: a plate 2 x 4 over the window and the
next course's top row, and on it two rows of curved slopes 2 x 1 (11477) back to back, so the roof rounds over from
side to side. (roof_dormer is the same box with a pointed gable of 45° slopes.) Returns that function.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["3005", "60592", "60601", "3010", "3020", "11477"]

DOWN = {"+x": 0, "+y": 1, "-x": 2, "-y": 3}


def build(m, x, y, z, color="White", cheeks="Black", roof="White", title="Dormer"):
    """The cheeks, the window, and the back brick; returns a function that adds the curved roof."""
    with m.step(f"{title}: window"):
        for r in range(2):
            m.add("3005", cheeks, at=(x, y, z + 3 * r))
            m.add("3005", cheeks, at=(x + 3, y, z + 3 * r))
        f = m.add("60592", color, at=(x + 1, y, z), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601",
            "Trans-Clear",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
        )
        m.add("3010", color, at=(x, y + 1, z), rot=1)

    def curved_roof():
        with m.step(f"{title}: roof"):
            m.add("3020", color, at=(x, y, z + 6), rot=1)
            for r in range(2):
                m.add("11477", roof, at=(x, y + r, z + 7), rot=DOWN["-x"])
                m.add("11477", roof, at=(x + 2, y + r, z + 7), rot=DOWN["+x"])

    return curved_roof


def example(m):
    """A short roof, two courses deep, with a curved dormer in the middle of the bottom course."""
    with m.step("Base"):
        m.add("3958", "Light Bluish Gray", at=(0, 0, 0))
    with m.step("Roof, course 1"):
        m.add("3040b", "Dark Red", at=(0, 0, 1), rot=3)
        m.add("3040b", "Dark Red", at=(5, 0, 1), rot=3)
        m.add("3009", "Dark Red", at=(0, 2, 1), rot=1)
    roof = m.use("curved_dormer", x=1, y=0, z=1)
    with m.step("Roof, course 2"):
        m.add("3037", "Dark Red", at=(0, 1, 4), rot=3)
        m.add("3039", "Dark Red", at=(4, 1, 4), rot=3)
    roof()
