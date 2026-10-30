# SPDX-License-Identifier: Apache-2.0
"""Feather shingles: a bird's plumage in bands of color, strips of small curved slopes built flat and hung on a studded
side so each band overlaps the one below like roof shingles, every other band shifted half a feather.

m.use("feather_shingles", x, y, z, length=6, colors=("Dark Green", "Green", "Bright Green"), facing="front", wall="Dark
Bluish Gray", title="Feathers") builds, on the studs at height z, a wall one stud thick and `length` studs long (an even
2 to 12) from (x, y), one course a band, each course bricks 1 x 2 x 1 2/3 with four studs on the side pointing out (5
plates, 2 studs tall), every other course starting with a brick 1 x 1 x 1 2/3 so the joints cross. On each course hangs
a band built flat: a plate 2 x `length` with a row of curved slopes 2 x 2 on it, falling downward once it's stood up, so
each band's thin lower edge tucks over the band below. Every other band starts with a curved slope 2 x 1 and ends with
another, which shifts its feathers half a feather (a stud). colors gives the bands, top first, one band each (1 to 6):
grade them from the back's color down to the belly's. facing is the side: "front" (-y) and "back" (+y) walls run along
x, "left" (-x) and "right" (+x) ones along y; (x, y) is the wall's first cell. Each band is a submodel. It brings its
own steps and returns the z of the wall's top.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43290. Not derived from any
model file."""

PARTS = [
    "22885",
    "32952",
    "15068",
    "11477",
    "3666",
    "3460",
    "4282",
    "3795",
    "3832",
    "3020",
    "3021",
    "3022",
]

# By facing: the way the wall runs, the quarter turn that points a side stud brick's studs out, the turn that stands a
# band (built flat along x, falling to -y) up against the face, and where the band's corner goes (relative to the
# wall's first cell, in studs, with the band's thickness, 3 plates, as `t`).
FACING = {
    "front": ("x", 3, Rot(x=90), lambda t: (0, -t)),
    "back": ("x", 1, Rot(x=90, z=180), lambda t: (0, 1)),
    "left": ("y", 2, Rot(x=90, z=-90), lambda t: (-t, 0)),
    "right": ("y", 0, Rot(x=90, z=90), lambda t: (1, 0)),
}


def _band(m, length, shifted, color):
    """A band lying flat along x: its plate, then the feathers on it, falling to -y (down, once it's stood up)."""
    with m.step("Plate"):
        m.row(Plate, color, at=(0, 0, 0), length=length, axis="x", width=2)
    with m.step("Feathers"):
        a = 0
        if shifted:
            m.add("11477", color, at=(0, 0, 1), rot=3)
            a = 1
        while a + 2 <= length:
            m.add("15068", color, at=(a, 0, 1), rot=3)
            a += 2
        if a < length:
            m.add("11477", color, at=(a, 0, 1), rot=3)


def build(
    m,
    x,
    y,
    z,
    length=6,
    colors=("Dark Green", "Green", "Bright Green"),
    facing="front",
    wall="Dark Bluish Gray",
    title="Feathers",
):
    """The studded wall a course at a time, then the bands, top first; returns the z of the wall's top."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if length % 2 or not 2 <= length <= 12:
        raise ValueError(
            f"feather_shingles takes an even length of 2 to 12, not {length}"
        )
    if not 1 <= len(colors) <= 6:
        raise ValueError(
            f"feather_shingles takes 1 to 6 band colors, not {len(colors)}"
        )
    axis, turn, stand, corner = FACING[facing]
    n = len(colors)

    def cell(a):
        return (x + a, y) if axis == "x" else (x, y + a)

    for k in range(n):
        zc = z + 5 * k
        with m.step(f"{title}: wall, course {k + 1}"):
            a = 0
            if k % 2:
                m.add("32952", wall, at=(*cell(0), zc), rot=turn)
                a = 1
            while a + 2 <= length:
                m.add("22885", wall, at=(*cell(a), zc), rot=turn)
                a += 2
            if a < length:
                m.add("32952", wall, at=(*cell(a), zc), rot=turn)
    for k in range(n):
        # The bands from the top: colors[0] on the top course.
        course = n - 1 - k
        band = m.submodel(
            "feather_band", _band, length=length, shifted=k % 2 == 1, color=colors[k]
        )
        t = (band.box[1][2] - band.box[0][2]) * 0.4
        dx, dy = corner(t)
        with m.step(f"{title}: band {k + 1}"):
            m.place(band, at=(x + dx, y + dy, z + 5 * course), rot=stand)
    return z + 5 * n


def example(m):
    """A bird's side, three bands of green on a dark wall."""
    with m.step("Base"):
        m.add("3034", "Dark Bluish Gray", at=(0, 0, 0), rot=1)  # Plate 2 x 8, along x
    m.use("feather_shingles", x=1, y=1, z=1, length=6)
    with m.step("Back's top"):
        m.add("3666", "Dark Green", at=(1, 1, 16), rot=1)  # Plate 1 x 6, along x
