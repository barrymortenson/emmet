# SPDX-License-Identifier: Apache-2.0
"""Color block facade: a front of narrow wall sections, each its own bright color with a window framed in another,
split by pilasters in a trim color, the cheerful storybook way.

m.use("color_block_facade", x, y, z, rows=4, sections=(("Bright Pink", 4, "Magenta"), ("Bright Light Yellow", 4,
"Medium Azure"), ("Lime", 4, "Dark Pink")), trim="White", title="Facade") builds a facade along x from (x, y, z),
facing front (-y), `rows` bricks tall: a pilaster (a column of 1 x 1 bricks in the trim color), then each section in
turn, `width` studs of wall in its color with a 1 x 2 x 2 window (window_frame) centered in it in its frame color,
then another pilaster. A course of plates in the trim color caps it all and ties the pilasters to the sections.
sections is a list of (color, width, frame color), widths 2 to 6. Returns the z of the top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43217. Not derived from any
model file."""

PARTS = [
    "3005",
    "3004",
    "3622",
    "3010",
    "60592",
    "60601",
    "3623",
    "3710",
    "3666",
    "3460",
]


def build(
    m,
    x,
    y,
    z,
    rows=4,
    sections=(
        ("Bright Pink", 4, "Magenta"),
        ("Bright Light Yellow", 4, "Medium Azure"),
        ("Lime", 4, "Dark Pink"),
    ),
    trim="White",
    title="Facade",
):
    """The sections, the pilasters, and the cap; returns the z of the top."""
    if rows < 3 or not sections or any(not 2 <= w <= 6 for _, w, _ in sections):
        raise ValueError(
            f"color_block_facade needs 3 rows or more and sections 2 to 6 studs wide, not {rows}, {sections}"
        )
    at = x
    for i, (color, width, frame) in enumerate(sections):
        with m.step(f"{title}, pilaster {i + 1}"):
            m.stack("3005", trim, at=(at, y, z), count=rows)
        at += 1
        m.use(
            "window_frame",
            x=at,
            y=y,
            z=z,
            length=width,
            rows=rows,
            offset=(width - 2) // 2,
            sill=1,
            window="1x2x2",
            color=color,
            frame_color=frame,
            title=f"{title}, section {i + 1}",
        )
        at += width
    with m.step(f"{title}, pilaster {len(sections) + 1}"):
        m.stack("3005", trim, at=(at, y, z), count=rows)
    with m.step(f"{title}: cap"):
        m.row(Plate, trim, at=(x, y, z + 3 * rows), length=at + 1 - x, first=3)
    return z + 3 * rows + 1


def example(m):
    """Three color blocks on a plate."""
    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, 0, 0), length=16, first=4)
    m.use("color_block_facade", x=0, y=0, z=1)
