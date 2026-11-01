# SPDX-License-Identifier: Apache-2.0
"""Garden gate: a hinged gate between two capped posts, with picket fence running off either side, the way into a
cottage garden.

m.use("garden_gate", x, y, z, left=1, right=1, axis="x", open=False, color="White", title="Gate", id=None) builds, on
the studs at height z (a baseplate is z = 0.5), along x (or y, with axis="y") from (x, y), one stud deep: `left` picket
fence panels (Fence 1 x 4 x 2 Picket, 33303), a post, the gate (Gate 1 x 4 x 2, 3186, on its hinge base, 3187), a post,
and `right` panels. Each post is two 1 x 1 bricks with a round plate and a round tile on top, two plates proud of the
fence, so the posts read as the gate's frame. open=True swings the gate a quarter turn on its hinge, out to the front
(-y; -x along y). left and right: 0 to 4 panels, each 4 studs. The whole run is 4 x (left + right) + 6 studs. With `id`,
the gate is `id` and its hinge base `<id>/hinge`, so the user can talk about it. It keeps clear the way through the gate
on both sides (3 studs out, from a plate above the ground up 5 bricks, a figure's headroom; docs/spec/building-rules.md,
rule 2), on the front 3 studs wide when the gate stands open there, so a bench, a lamp, or a planter in the path is a
figure_clearance finding. Returns the z of the posts' top, with .door_top (the top of that headroom, which an arch or an
arbor over the gate starts at or above) and .door ((x0, y0), (x1, y1), the opening's footprint between the posts, where
a path goes).
"""

from emmet_dsl.core import APPROACH, HEADROOM, fit

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43217. Not derived from any
model file."""

PARTS = ["33303", "3187", "3186", "3005", "6141", "98138"]


def build(
    m,
    x,
    y,
    z,
    left=1,
    right=1,
    axis="x",
    open=False,
    color="White",
    title="Gate",
    id=None,
):
    """The fence panels, posts, and gate; returns the z of the posts' top, with .door_top and .door."""
    if not 0 <= left <= 4 or not 0 <= right <= 4 or axis not in ("x", "y"):
        raise ValueError(
            f"garden_gate takes 0 to 4 panels a side and an axis of x or y, not {left}, {right}, {axis!r}"
        )

    def add(part, s, h, rot=0, id=None):
        """A part s studs along the run, h plates up; along y, every part turns a quarter more."""
        if axis == "x":
            return m.add(part, color, at=(x + s, y, z + h), rot=rot, id=id)
        return m.add(part, color, at=(x, y + s, z + h), rot=rot + 1, id=id)

    gate = 4 * left + 1
    with m.step(f"{title}, posts"):
        for s in (gate - 1, gate + 4):
            add("3005", s, 0)
            add("3005", s, 3)
    with m.step(f"{title}, gate"):
        add("3187", gate, 0, id=f"{id}/hinge" if id else None)
        if open:
            # The hinge's axis is the middle of the gate's last stud; a quarter turn about it swings the gate out.
            if axis == "x":
                m.add("3186", color, at=(x + gate + 3, y - 3, z + 1), rot=1, id=id)
            else:
                m.add("3186", color, at=(x - 3, y + gate + 3, z + 1), rot=0, id=id)
        else:
            add("3186", gate, 1, id=id)
    for side, (start, n) in (("left", (0, left)), ("right", (gate + 5, right))):
        if n:
            with m.step(f"{title}, {side} fence"):
                for k in range(n):
                    add("33303", start + 4 * k, 0, 1)
    with m.step(f"{title}, post caps"):
        for s in (gate - 1, gate + 4):
            add("6141", s, 6)
            add("98138", s, 7)
    # The way through, in (along the run, across it) studs: the open gate stands in the last stud of the front space.
    lo, hi = z + 1, z + HEADROOM
    front = 3 if open else 4
    spaces = [
        ((gate, -APPROACH), (gate + front, 0)),
        ((gate, 1), (gate + 4, 1 + APPROACH)),
    ]
    opening = ((gate, 0), (gate + 4, 1))

    def world(a, b):
        return (x + a, y + b) if axis == "x" else (x + b, y + a)

    m.keep_clear(
        [((*world(*p0), lo), (*world(*p1), hi)) for p0, p1 in spaces],
        f"a figure walking through {title}",
    )
    return fit(z + 8, door_top=hi, door=(world(*opening[0]), world(*opening[1])))


def example(m):
    """A gate standing open between two panels of picket fence."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("garden_gate", x=1, y=8, z=0.5, left=1, right=1, open=True)
