# SPDX-License-Identifier: Apache-2.0
"""Start gantry: a race's start and finish arch, two pillars faced with sign panels on round bases, carrying a beam
with a banner of checkered flags either side of a round badge.

m.use("start_gantry", x, y, z, span=6, courses=3, color="Yellow", trim="Red", banner="Yellow", title="Gantry") stands,
on the studs at height z, two pillars 2 x 2, the left over (x + 1, y + 1) to (x + 3, y + 3) and the right one `span`
studs further right, so a track can run between them along y, each on a round plate 4 x 4 (60474) in `trim`. A pillar
is `courses` courses of two bricks 1 x 2 x 1 2/3 with studs on one side (22885) back to back, a 2 x 2 tile on the
front and back of each as the signs, and a 2 x 2 plate in `trim` between courses. Across the pillar tops goes a 2 x
(span + 4) plate and on it a banner one course tall: bricks 22885 facing front with a round tile 2 x 2 (14769) in
`trim` in the middle (a stud right of it when span / 2 is even), checkered 1 x 1 tiles (3070bp1c) at both ends, and
plain 2 x 2 tiles between, with plain bricks behind. span is even, 2 to 12. Returns the z of the banner's top.
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72036. Not derived from any
model file."""

PARTS = ["60474", "22885", "3068b", "3022", "14769", "3070bp1c", "4282", "3004"]

FRONT, BACK = Rot(x=90), Rot(x=-90)  # a tile on its back, its face to -y, or to +y
OUT = 0.4  # a tile's thickness, in studs
COURSE = 6  # a pillar course: a brick 1 x 2 x 1 2/3 and a plate, in plates
PLATES = {
    16: "4282",
    14: "3832",
    12: "2445",
    10: "3832",
    8: "3034",
    6: "3795",
}  # plates 2 x n


def build(
    m,
    x,
    y,
    z,
    span=6,
    courses=3,
    color="Yellow",
    trim="Red",
    banner="Yellow",
    title="Gantry",
):
    """The pillars, the beam, and the banner; returns the z of the banner's top."""
    if span % 2 or not 2 <= span <= 12:
        raise ValueError(f"start_gantry takes an even span from 2 to 12, not {span}")
    if courses < 1:
        raise ValueError(f"start_gantry needs 1 or more courses, not {courses}")
    pillars = (("left", x + 1), ("right", x + 3 + span))
    for side, px in pillars:
        with m.step(f"{title}: {side} base"):
            m.add("60474", trim, at=(px - 1, y, z))
    for c in range(courses):
        zc = z + 1 + COURSE * c
        for side, px in pillars:
            with m.step(f"{title}: {side} pillar, course {c + 1}"):
                if c:
                    m.add("3022", trim, at=(px, y + 1, zc - 1))
                m.add("22885", color, at=(px, y + 1, zc), rot=3)
                m.add("22885", color, at=(px, y + 2, zc), rot=1)
                m.add("3068b", color, at=(px, y + 1 - OUT, zc), rot=FRONT)
                m.add("3068b", color, at=(px, y + 3, zc), rot=BACK)
    zb = z + 1 + COURSE * courses - 1
    length = span + 4
    with m.step(f"{title}: beam"):
        m.add(PLATES[length], trim, at=(x + 1, y + 1, zb), rot=1)
    bricks = length // 2
    with m.step(f"{title}: banner, front bricks"):
        for k in range(bricks):
            m.add("22885", banner, at=(x + 1 + 2 * k, y + 1, zb + 1), rot=3)
    with m.step(f"{title}: banner, back bricks"):
        m.row(Brick, banner, at=(x + 1, y + 2, zb + 1), length=length, sizes=(2,))
    with m.step(f"{title}: banner, flags"):
        for bx in (x + 1, x + length - 1):
            for dx in (0, 1):
                for dz in (0, 2.5):
                    m.add(
                        "3070bp1c",
                        "White",
                        at=(bx + dx, y + 1 - OUT, zb + 1 + dz),
                        rot=FRONT,
                    )
    with m.step(f"{title}: banner, badge"):
        for k in range(1, bricks - 1):
            part, tint = ("14769", trim) if k == bricks // 2 else ("3068b", banner)
            m.add(part, tint, at=(x + 1 + 2 * k, y + 1 - OUT, zb + 1), rot=FRONT)
    return zb + 6


def example(m):
    """A gantry over a strip of track."""
    from emmet_dsl import Plate

    with m.step("Track"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(12, 4))
    m.use("start_gantry", x=0, y=0, z=1, span=6)
