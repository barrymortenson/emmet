# SPDX-License-Identifier: Apache-2.0
"""Float on clear: a thin stand of clear parts that holds a game object up in mid-air, so it seems to hover.

m.use("float_on_clear", x, y, z, height=2, kind="column", color="Trans-Clear", title="Stand") builds, on the studs at
height z, a stand `height` bricks tall whose top is where the floating thing goes. kind: "column" (a stack of round
bricks 1 x 1 (3062b) on the stud at (x, y), topped by its one stud: the thinnest stand, for a coin, a cloud, a star),
or "panels" (pairs of panels 1 x 2 x 2 (4864b) back to back over (x, y) to (x + 2, y + 2), two bricks a pair, so
height is rounded up to even: a stiffer stand with a 2 x 2 top, for a block or anything heavy). Keep the stand under
the middle of what it carries and behind it from the main view. Returns the z of the stand's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72036. Not derived from any
model file."""

PARTS = ["3062b", "4864b"]

KINDS = ("column", "panels")


def build(m, x, y, z, height=2, kind="column", color="Trans-Clear", title="Stand"):
    """The stand; returns the z of its top."""
    if kind not in KINDS:
        raise ValueError(
            f"float_on_clear: kind must be one of {', '.join(KINDS)}, not {kind!r}"
        )
    if height < 1:
        raise ValueError(
            f"float_on_clear needs a height of 1 or more bricks, not {height}"
        )
    if kind == "column":
        for k in range(0, height, 4):
            n = min(4, height - k)
            with m.step(f"{title}" + (f" {k // 4 + 1}" if height > 4 else "")):
                m.stack("3062b", color, at=(x, y, z + 3 * k), count=n)
        return z + 3 * height
    pairs = (height + 1) // 2
    for k in range(pairs):
        with m.step(f"{title}" + (f" {k + 1}" if pairs > 1 else "")):
            # The panels' walls meet in the middle, back to back.
            m.add("4864b", color, at=(x, y, z + 6 * k), rot=2)
            m.add("4864b", color, at=(x + 1, y, z + 6 * k), rot=0)
    return z + 6 * pairs


def _cloud(m, x, y, z):
    """A small white cloud on a 2 x 6 plate whose middle is at (x + 3, y + 1)."""
    with m.step("Cloud"):
        m.add("3795", "White", at=(x, y, z), rot=1)
        m.add("15068", "White", at=(x, y, z + 1), rot=2)
        m.add("3003", "White", at=(x + 2, y, z + 1))
        m.add("15068", "White", at=(x + 4, y, z + 1), rot=0)
    with m.step("Cloud top"):
        m.add("15068", "White", at=(x + 2, y, z + 4), rot=2)


def example(m):
    """A cloud on a clear column and another on clear panels, over a patch of grass."""
    from emmet_dsl import Plate

    with m.step("Grass"):
        m.row(Plate, "Bright Green", at=(0, 0, 0), length=8, width=6)
    top = m.use("float_on_clear", x=3, y=1, z=1, height=3)
    _cloud(m, 0, 0, top)
    top = m.use("float_on_clear", x=4, y=4, z=1, height=4, kind="panels")
    _cloud(m, 2, 4, top)
