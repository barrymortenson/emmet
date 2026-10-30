# SPDX-License-Identifier: Apache-2.0
"""Window band: the long run of windows down a passenger car's or bus's side, windows side by side over a wall with a
colored stripe, a plate course over them, and dark posts where the doors go.

m.use("window_band", x, y, z, length=12, color="White", stripe="Red", post="Black", doors=(), title="Window band")
builds one side 1 stud thick, `length` studs along x from (x, y, z): a course of bricks, a course of plates in the
stripe color, then windows 1 x 2 x 2 with their glass (60592c01) side by side, the frames themselves the pillars
between the panes, and a course of plates over them that ties the band together. doors: the numbers of the windows
(0 from the front) whose bay is a door: two dark 1 x 2 bricks stand there instead, the door's posts for the door the
car's builder hangs. Even lengths. Returns the z of the top plates' top.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 60337. Not derived from any
model file."""

PARTS = ["60592c01", "3004", "3009", "3008", "3666", "3460"]


def build(
    m,
    x,
    y,
    z,
    length=12,
    color="White",
    stripe="Red",
    post="Black",
    doors=(),
    title="Window band",
):
    """The wall course, the stripe, the windows, and the plates over them; returns the z of the top."""
    if length < 2 or length % 2:
        raise ValueError(f"window_band needs an even length, not {length}")
    with m.step(f"{title}: wall"):
        m.row(Brick, color, at=(x, y, z), length=length, axis="x")
    with m.step(f"{title}: stripe"):
        m.row(Plate, stripe, at=(x, y, z + 3), length=length, axis="x")
    n = length // 2
    for s in range(0, n, 4):
        with m.step(f"{title}: windows {s // 4 + 1}"):
            for i in range(s, min(s + 4, n)):
                if i in doors:
                    m.stack("3004", post, at=(x + 2 * i, y, z + 4), count=2, rot=1)
                else:
                    m.add("60592c01", color, at=(x + 2 * i, y, z + 4), rot=1)
    with m.step(f"{title}: top plates"):
        # Starting a stud in, so the plates' joints miss the windows'.
        m.row(
            Plate,
            color,
            at=(x, y, z + 10),
            length=length,
            axis="x",
            first=1 if length > 2 else None,
        )
    return z + 11


def example(m):
    """Both sides of a 6-wide car 12 long, the third bay on the left a door."""
    with m.step("Floor"):
        m.add("3028", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("window_band", x=0, y=0, z=1, doors=(2,), title="Left side")
    m.use("window_band", x=0, y=5, z=1, title="Right side")
