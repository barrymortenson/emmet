# SPDX-License-Identifier: Apache-2.0
"""Portcullis: a gate grille that slides up and down in a slot inside a thick wall, its ends running in the wall to
either side of the gateway.

m.use("portcullis", x, y, z, length=8, width=4, grilles=1, color="Light Bluish Gray", grille="Dark Bluish Gray",
title="Portcullis") builds a wall 3 studs thick and `length` studs long along x from (x, y), from z: a front leaf (row
y), a slot (row y + 1) and a back leaf (row y + 2). The gateway through the leaves is width - 2 studs wide in the
middle, 2 x `grilles` bricks tall, under an arch (Arch 1 x 4 over 2, Arch 1 x 8 x 2 over 6) in each leaf. The slot is
open `width` studs wide from the floor to the top, a stud wider than the gateway each side, and the grille stands in it:
spindled fences 1 x 4 x 2 (30055), `width` / 4 across, for the bars, and a plate 1 x `width` on them for the cross rail
that joins them, `grilles` times over; its ends are hidden in the wall, so it can only slide straight up (its top rides
a plate above the gateway, into the open slot). A course of plates ties the three rows at the top and leaves the slot
open, for the grille to be raised into a room above. width is 4 or 8. Returns the z of the top.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "30055",
    "3659",
    "3308a",
    "3710",
    "3460",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "4217",
    "2465",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "91988",
    "4282",
    "3024",
    "3023b",
    "3623",
    "3666",
    "4477",
    "60479",
]

ARCHES = {
    4: ("3659", 1),
    8: ("3308a", 2),
}  # the arch over the gateway, and its height in rows
TOPS = {4: "3710", 8: "3460"}  # Plate 1 x width, joining the grille's lattices


def build(
    m,
    x,
    y,
    z,
    length=8,
    width=4,
    grilles=1,
    color="Light Bluish Gray",
    grille="Dark Bluish Gray",
    title="Portcullis",
):
    """The three rows a course at a time, the grille, and the tie course; returns the z of the top."""
    if width not in ARCHES or grilles not in (1, 2):
        raise ValueError(
            f"portcullis takes a width of 4 or 8 and 1 or 2 grilles, not {width}, {grilles}"
        )
    if length < width + 2:
        raise ValueError(
            f"portcullis needs a length of width + 2 ({width + 2}) or more, not {length}"
        )
    o = (
        length - width
    ) // 2  # where the slot starts; the gateway is a stud in from each side of it
    arch, arch_rows = ARCHES[width]
    gate_rows = 2 * grilles
    rows = gate_rows + arch_rows + 1
    for r in range(rows):
        zr = z + 3 * r
        if r < gate_rows:
            leaf_gap = (o + 1, o + width - 1)
        elif r < gate_rows + arch_rows:
            leaf_gap = (o, o + width)
        else:
            leaf_gap = None
        for row, gap, part in (
            (y + 2, leaf_gap, "back"),
            (y + 1, (o, o + width), "slot"),
            (y, leaf_gap, "front"),
        ):
            with m.step(f"{title}, row {r + 1}: {part}"):
                if part != "slot" and r == gate_rows:
                    m.add(arch, color, at=(x + o, row, zr), rot=1)
                runs = [(0, gap[0]), (gap[1], length)] if gap else [(0, length)]
                for start, end in runs:
                    n = end - start
                    if n > 0:
                        first = max(1, n // 2) if r % 2 and n > 2 else None
                        m.row(
                            Brick, color, at=(x + start, row, zr), length=n, first=first
                        )
        if r == gate_rows - 1:
            # The grille goes in once the slot's sides are up, before the arches close the gateway over it.
            with m.step(f"{title}: grille"):
                for k in range(grilles):
                    for u in range(0, width, 4):
                        m.add("30055", grille, at=(x + o + u, y + 1, z + 7 * k), rot=1)
                    m.add(TOPS[width], grille, at=(x + o, y + 1, z + 7 * k + 6), rot=1)
    top = z + 3 * rows
    with m.step(f"{title}: tie course"):
        # Two-wide plates over the front and slot rows at the left and the slot and back rows at the right, and a
        # long plate on the other row of each, so the course ties all three rows and leaves the slot open.
        m.row(Plate, color, at=(x, y, top), length=o, width=2)
        m.row(
            Plate,
            color,
            at=(x + o + width, y + 1, top),
            length=length - o - width,
            width=2,
        )
        m.row(Plate, color, at=(x + o, y, top), length=length - o)
        m.row(Plate, color, at=(x, y + 2, top), length=o + width)
    return top + 1


def example(m):
    """A gateway 2 studs wide with its portcullis down."""
    m.use("portcullis", x=0, y=0, z=0, length=6, grille="Reddish Brown")
