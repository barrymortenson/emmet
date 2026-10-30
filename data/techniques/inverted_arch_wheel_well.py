# SPDX-License-Identifier: Apache-2.0
"""Inverted arch wheel well: a clean, round-topped wheel arch in a large car's side, an arch 1 x 6 hung under a long
plate over each wheel, the strip that forms the side's lower edge.

m.use("inverted_arch_wheel_well", x, y, z, length=12, wheels=(6,), raised=False, color="Red", title="Wheel well") lays a
course `length` studs along x from (x, y, z), one stud deep: an arch 1 x 6 (3455, or the raised arch 92950 with
raised=True) centered on each of `wheels` (studs from x, at least 3 from either end and 6 apart), bricks between, and
plates 1 x n over the whole course, which hold the arches by their tops. The arch's opening is 4 studs wide and about 2
plates high at its middle, so a tire 3.8 studs across under it clears it with its top up to z + 1.75. Put the course on
piers or the body either side; the body builds up from its top. Returns the z of the plates' top (z + 4).
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10300. Not derived from any
model file."""

PARTS = [
    "3455",
    "92950",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
]


def build(
    m,
    x,
    y,
    z,
    length=12,
    wheels=(6,),
    raised=False,
    color="Red",
    title="Wheel well",
):
    """The course with its arches, then the plates over it; returns the z of the top."""
    centers = sorted(wheels)
    if any(c < 3 or c > length - 3 for c in centers) or any(
        b - a < 6 for a, b in zip(centers, centers[1:], strict=False)
    ):
        raise ValueError(
            f"inverted_arch_wheel_well needs each wheel 3 or more from the ends and 6 apart, not {list(wheels)}"
        )
    arch = "92950" if raised else "3455"
    with m.step(f"{title}: arches"):
        a = 0
        for c in centers:
            if c - 3 > a:
                m.row(Brick, color, at=(x + a, y, z), length=c - 3 - a)
            m.add(arch, color, at=(x + c - 3, y, z), rot=1)
            a = c + 3
        if length > a:
            m.row(Brick, color, at=(x + a, y, z), length=length - a)
    with m.step(f"{title}: plates"):
        m.row(Plate, color, at=(x, y, z + 3), length=length)
    return z + 4


def example(m):
    """A side over a wheel, on two piers."""
    with m.step("Piers"):
        for px in (0, 9):
            m.stack(Brick(1, 3), "Dark Bluish Gray", at=(px, 0, 0), count=2, rot=1)
            m.stack(Plate(1, 3), "Dark Bluish Gray", at=(px, 0, 6), count=2, rot=1)
    with m.step("Axle"):
        m.add("3010", "Dark Bluish Gray", at=(4, 2, 0), rot=1)
        m.add("3701", "Light Bluish Gray", at=(4, 2, 3), rot=1)
    with m.step("Wheel"):
        m.add_raw("2780", "Black", [40, -38, -120, 1, 0, 0, 0, 1, 0, 0, 0, 1])
        for part, color in (("55981", "Metallic Silver"), ("30391", "Black")):
            m.add_raw(part, color, [20, -38, -120, 0, 0, 1, 0, 1, 0, -1, 0, 0])
    m.use("inverted_arch_wheel_well", x=0, y=0, z=8)
