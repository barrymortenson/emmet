# SPDX-License-Identifier: Apache-2.0
"""Module link block: blocks set across the seam of two terrain modules, locking them into one scene and reading as
one more grass block.

m.use("module_link_block", x, y, z, length, axis="y", links=2, wide=True, dirt="Reddish Brown", grass="Bright Green",
title="Links") bridges the seam that runs `length` studs along `axis` from (x, y), on the modules' studs at height z
(the seam is the line x = x for axis "y", y = y for axis "x"). It sets `links` blocks along it, spread evenly on the
2-stud grid: each a brick in the dirt color whose studs-under bite both modules, a stud each side for a 1 x 2 or two
for a 2 x 2 (wide), with a plate on top in the grass color: a 2 x 2 jumper on the wide block, so it shows one stud
like any block, or a 1 x 2 plate on the narrow one. Swap the colors for a sand (Tan) or stone (Light Bluish Gray)
seam. Returns the z of the links' top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3003", "87580", "3004", "3023b"]


def build(
    m,
    x,
    y,
    z,
    length,
    axis="y",
    links=2,
    wide=True,
    dirt="Reddish Brown",
    grass="Bright Green",
    title="Links",
):
    """The links along the seam; returns the z of their top."""
    if axis not in ("x", "y"):
        raise ValueError(f"module_link_block takes an axis of 'x' or 'y', not {axis!r}")
    cells = length // 2
    if not 1 <= links <= cells:
        raise ValueError(
            f"module_link_block fits 1 to {cells} links on a {length}-stud seam, not {links}"
        )
    with m.step(title):
        for k in range(links):
            c = 2 * int((k + 0.5) * cells / links)  # this link's cell along the seam
            at = (x - 1, y + c, z) if axis == "y" else (x + c, y - 1, z)
            if wide:
                m.add("3003", dirt, at=at)
                m.add("87580", grass, at=(at[0], at[1], z + 3))
            else:
                # A 1 x 2 across the seam (along x for a seam along y), on the first row of its cell.
                turn = 1 if axis == "y" else 0
                m.add("3004", dirt, at=at, rot=turn)
                m.add("3023b", grass, at=(at[0], at[1], z + 3), rot=turn)
    return z + 4


def example(m):
    """Two small modules side by side, locked by two links across their seam."""
    for i, (top, dirt) in enumerate(
        (("Bright Green", "Reddish Brown"), ("Tan", "Reddish Brown"))
    ):
        with m.step(f"Module {i + 1}"):
            for lx in (0, 6):
                for ly in (0, 6):
                    m.add("3003", dirt, at=(8 * i + lx, ly, 0))
            m.add("41539", top, at=(8 * i, 0, 3))
    m.use("module_link_block", x=8, y=0, z=4, length=8, links=2)
