# SPDX-License-Identifier: Apache-2.0
"""Stick torch: a thin wooden stick with a glowing square flame on top, the blocky-world torch.

m.use("stick_torch", x, y, z, height=2, centered=True, stick="Reddish Brown", seat="Light Bluish Gray",
flame=("Trans Yellow", "Trans Orange"), title="Torch") stands a torch on the studs at height z. centered: the torch
stands in the middle of the 2 x 2 block whose front-left corner is (x, y), on a 2 x 2 jumper in the seat color (match
it to the block), as the style puts every thin thing; otherwise it stands on the stud at (x, y). The torch is a 1 x 1
round plate and `height` 1 x 1 round bricks in the stick color, then the flame: a square 1 x 1 plate in each of
`flame`, bottom up. Returns the z of the flame's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["87580", "6141", "3062b", "3024"]


def build(
    m,
    x,
    y,
    z,
    height=2,
    centered=True,
    stick="Reddish Brown",
    seat="Light Bluish Gray",
    flame=("Trans Yellow", "Trans Orange"),
    title="Torch",
):
    """The seat, the stick, the flame; returns the z of the top."""
    if not 1 <= height <= 4 or not flame:
        raise ValueError(
            f"stick_torch takes a height of 1 to 4 round bricks and at least one flame color, not {height}, {flame}"
        )
    with m.step(title):
        if centered:
            m.add("87580", seat, at=(x, y, z))
            x, y, z = x + 0.5, y + 0.5, z + 1
        m.add("6141", stick, at=(x, y, z))
        m.stack("3062b", stick, at=(x, y, z + 1), count=height)
        zf = z + 1 + 3 * height
        for k, color in enumerate(flame):
            m.add("3024", color, at=(x, y, zf + k))
    return zf + len(flame)


def example(m):
    """A torch centered on a stone block, and one on the grass beside it."""
    with m.step("Ground"):
        m.add("3020", "Bright Green", at=(0, 0, 0), rot=1)
    with m.step("Stone block"):
        m.add("3003", "Light Bluish Gray", at=(0, 0, 1))
        m.add("3022", "Dark Bluish Gray", at=(0, 0, 4))
    m.use("stick_torch", x=0, y=0, z=5)
    m.use("stick_torch", x=3, y=1, z=1, height=1, centered=False, title="Ground torch")
