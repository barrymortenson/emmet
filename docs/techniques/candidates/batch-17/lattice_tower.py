# SPDX-License-Identifier: Apache-2.0
"""Lattice tower: a tall open steel tower of stacked modules, each a deck, corner posts, X-braced girder faces, and a
ribbed core.

m.use("lattice_tower", x, y, z, levels=3, core=4, color="Red", core_color="Dark Bluish Gray",
deck="Light Bluish Gray", title="Tower") builds a tower 8 x 8 studs from (x, y), on the studs at z, of `levels`
identical modules 16 plates tall: a Plate 8 x 8 as the deck; a brick 1 x 1 x 5 (2453b) standing at each corner; a
girder panel 1 x 6 x 5 (64448), braced with an X, on each face between the posts, so the tower reads as open
steelwork; and a core in the middle, `core` studs square (2 or 4), of bricks 1 x 2 with a grille (2877), five
courses, their joints crossed course to course. A last deck caps the tower. The modules go on a step at a time, so
the tower can be stopped at any level. Returns the z of the top deck's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10341. Not derived from any
model file."""

PARTS = ["41539", "2453b", "64448", "2877"]

HEIGHT = 15  # a module's posts and girders, in plates


def _core_course(m, cx, cy, z, size, r, color):
    """One course of the grille core, `size` square from (cx, cy), turned course to course so the joints cross."""
    if size == 2:
        if r % 2:
            m.add("2877", color, at=(cx, cy, z), rot=1)
            m.add("2877", color, at=(cx, cy + 1, z), rot=1)
        else:
            m.add("2877", color, at=(cx, cy, z))
            m.add("2877", color, at=(cx + 1, cy, z))
        return
    # A ring 4 x 4: two full runs and two short ones, swapped course to course.
    if r % 2 == 0:
        for yy in (cy, cy + 3):
            m.add("2877", color, at=(cx, yy, z), rot=1)
            m.add("2877", color, at=(cx + 2, yy, z), rot=1)
        m.add("2877", color, at=(cx, cy + 1, z))
        m.add("2877", color, at=(cx + 3, cy + 1, z))
    else:
        for xx in (cx, cx + 3):
            m.add("2877", color, at=(xx, cy, z))
            m.add("2877", color, at=(xx, cy + 2, z))
        m.add("2877", color, at=(cx + 1, cy, z), rot=1)
        m.add("2877", color, at=(cx + 1, cy + 3, z), rot=1)


def build(
    m,
    x,
    y,
    z,
    levels=3,
    core=4,
    color="Red",
    core_color="Dark Bluish Gray",
    deck="Light Bluish Gray",
    title="Tower",
):
    """The modules and the top deck; returns the z of the top."""
    if not 1 <= levels <= 8 or core not in (2, 4):
        raise ValueError(
            f"lattice_tower takes 1 to 8 levels and a core of 2 or 4, not {levels}, {core}"
        )
    c = (8 - core) // 2
    zl = z
    for k in range(levels):
        name = f"{title}, level {k + 1}"
        with m.step(f"{name}: deck and posts"):
            m.add("41539", deck, at=(x, y, zl))
            for px, py in ((0, 0), (7, 0), (0, 7), (7, 7)):
                m.add("2453b", color, at=(x + px, y + py, zl + 1))
        for r in range(5):
            with m.step(f"{name}: core, course {r + 1}"):
                _core_course(m, x + c, y + c, zl + 1 + 3 * r, core, r, core_color)
        with m.step(f"{name}: front and back girders"):
            m.add("64448", color, at=(x + 1, y, zl + 1), rot=1)
            m.add("64448", color, at=(x + 1, y + 7, zl + 1), rot=1)
        with m.step(f"{name}: side girders"):
            m.add("64448", color, at=(x, y + 1, zl + 1))
            m.add("64448", color, at=(x + 7, y + 1, zl + 1))
        zl += 1 + HEIGHT
    with m.step(f"{title}, top deck"):
        m.add("41539", deck, at=(x, y, zl))
    return zl + 1


def example(m):
    """Two levels of tower with a small core."""
    m.use("lattice_tower", x=0, y=0, z=0, levels=2, core=2)
