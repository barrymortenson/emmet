# SPDX-License-Identifier: Apache-2.0
"""Lift-off floor: an upper floor built on its own, two layers of plates with their seams crossed, with an opening for
the stairs, which sets on the walls below and lifts off.

m.use("lift_off_floor", length=8, depth=8, opening=(2, 4), color="Dark Bluish Gray", title="Floor") returns a submodel:
a length x depth floor whose lower layer runs its plates along x and upper layer along y, so no seam crosses another,
and which leaves an opening (w, d) studs at its back-right corner for the stairs. Place it on the walls with m.place.
"""

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350, set 60440. Not derived
from any model file."""

PARTS = [
    "3034",
    "3832",
    "3795",
    "3020",
    "3022",
    "3021",
    "3710",
    "3666",
    "3460",
    "3623",
    "3023b",
]

PLATES = {8: "3034", 6: "3795", 4: "3020", 2: "3022"}  # Plate 2 x n


def _strip(s, color, at, n, axis):
    """A 2-wide strip of n studs along axis from at, the longest plates first."""
    x, y, z = at
    for size in sorted(PLATES, reverse=True):
        while n >= size:
            s.add(PLATES[size], color, at=(x, y, z), rot=1 if axis == "x" else 0)
            x, y = (x + size, y) if axis == "x" else (x, y + size)
            n -= size


def build(
    m, length=8, depth=8, opening=(2, 4), color="Dark Bluish Gray", title="Floor"
):
    """The floor as a submodel."""
    ow, od = opening
    if length % 2 or depth % 2 or ow % 2 or od % 2:
        raise ValueError(
            f"lift_off_floor needs even sizes, not {length} x {depth} with {ow} x {od}"
        )

    def floor(s):
        with s.step("Lower layer"):
            for y in range(0, depth, 2):
                n = length - ow if y >= depth - od else length
                _strip(s, color, (0, y, 0), n, "x")
        with s.step("Upper layer"):
            for x in range(0, length, 2):
                n = depth - od if x >= length - ow else depth
                _strip(s, color, (x, 0, 1), n, "y")

    return m.submodel(title, floor)


def example(m):
    """A floor set on four low walls."""
    m.wall(0, 0, 0, length=8, depth=8, rows=2, color="White")
    floor = m.use("lift_off_floor")
    with m.step("Floor on"):
        m.place(floor, at=(0, 0, 6))
