# SPDX-License-Identifier: Apache-2.0
"""Beam spine floor: a light, stiff spine down a large car's middle, two rows of Technic beams on edge with their joints
staggered, pinned between two rows of Technic bricks that carry the floor plates.

m.use("beam_spine_floor", x, y, z, length=24, color="Light Bluish Gray", beam="Yellow", floor="Dark Bluish Gray",
pin="Blue", title="Spine") builds, along x from (x, y, z), a spine 4 studs wide: Technic bricks 1 x 4 (3701) end to end
on y and y + 3, and between them two rows of Technic beams standing on edge (holes along y), each row two or three beams
end to end, the second row's joints away from the first's so each beam laps a joint. Long pins (6558) through each
brick's holes run through both rows of beams, alternating sides, so the whole is one piece. Plates 8 x 8 (41539) across
the bricks' tops, 2 studs out each side, make the floor. length: 16, 24, or 32 studs. The beams mark the sub-assembly in
a bright color the way the books do; it doesn't show once the body is on. floor=None leaves the floor off, for a floor
of your own. Returns the z of the floor's top (z + 4), or of the bricks' top without one.
"""

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11381. Not derived from any
model file."""

PARTS = ["3701", "32524", "32278", "40490", "6558", "41539"]

BEAMS = {7: "32524", 9: "40490", 15: "32278"}  # Technic beam n
ROWS = {
    16: ([15], [7, 7]),
    24: ([15, 7], [7, 15]),
    32: ([15, 15], [7, 15, 9]),
}  # each row's beams, in holes
# A beam turned a quarter about x: on edge, its holes along y.
ON_EDGE = frame.rotation(x=90)


def _put(m, part, color, point, r=None):
    """Places a part whose origin is its center (a beam, a pin) at `point` (DSL), turned by r (DSL axes)."""
    r = frame.to_ldraw_rotation(r or frame.rotation())
    return m.add_raw(part, color, [*frame.to_ldraw_point(point), *r[0], *r[1], *r[2]])


def build(
    m,
    x,
    y,
    z,
    length=24,
    color="Light Bluish Gray",
    beam="Yellow",
    floor="Dark Bluish Gray",
    pin="Blue",
    title="Spine",
):
    """The bricks, the beams, the pins, and the floor; returns the z of the floor's top."""
    if length not in ROWS:
        raise ValueError(
            f"beam_spine_floor takes a length of 16, 24, or 32, not {length}"
        )
    zc = z + 1.75  # the holes' center line; a hole is on the stud line x + k
    with m.step(f"{title}: bricks, left"):
        for a in range(0, length, 4):
            m.add("3701", color, at=(x + a, y, z), rot=1)
    for k, row in enumerate(ROWS[length]):
        with m.step(f"{title}: beams, row {k + 1}"):
            first = 1
            for n in row:
                _put(
                    m,
                    BEAMS[n],
                    beam,
                    (x + first + (n - 1) / 2, y + 1.5 + k, zc),
                    ON_EDGE,
                )
                first += n
    with m.step(f"{title}: bricks, right"):
        for a in range(0, length, 4):
            m.add("3701", color, at=(x + a, y + 3, z), rot=1)
    # Pins from the left through the bricks' outer holes, from the right through their middle ones.
    starts = range(0, length, 4)
    halves = (starts[: len(starts) // 2], starts[len(starts) // 2 :])
    for k, half in enumerate(halves):
        with m.step(f"{title}: pins, left, {'front' if k == 0 else 'back'}"):
            for a in half:
                for h in (1, 3):
                    _put(m, "6558", pin, (x + a + h, y + 1.5, zc))
    with m.step(f"{title}: pins, right"):
        for a in starts:
            _put(m, "6558", pin, (x + a + 2, y + 2.5, zc))
    if floor is None:
        return z + 3
    with m.step(f"{title}: floor"):
        for a in range(0, length, 8):
            m.add("41539", floor, at=(x + a, y - 2, z + 3))
    return z + 4


def example(m):
    """A spine 24 studs long, with one floor plate on its back end so the spine shows."""
    top = m.use("beam_spine_floor", x=0, y=0, z=0, floor=None)
    with m.step("Floor"):
        m.add("41539", "Dark Bluish Gray", at=(16, -2, top))
