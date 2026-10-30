# SPDX-License-Identifier: Apache-2.0
"""Curb rumble strip: a race track's striped edge curb, curved slopes in two colors in turn, with a low rail standing
behind it on brackets.

m.use("curb_rumble_strip", x, y, z, length=8, facing=3, base="Light Bluish Gray", stripes=("Red", "White"),
rail="Blue", title="Curb") lays, on the studs at height z, a strip `length` studs long (even, 2 to 16) and 3 deep: a
layer of plates, then along its front two rows curved slopes 2 x 2 (15068) in the stripe colors in turn, falling to
the front, and along its back row brackets 1 x 2 - 1 x 2 (99780) facing back, their side studs carrying the rail: 1 x
N plates standing on edge, a stud tall, just behind the strip. It's a submodel, so a curb of one length and colors is
one file however many line the track; facing is the side the slopes fall to, in quarter turns from +x (3, the default,
is front, -y, the strip running along x from (x, y) to (x + length, y + 3) and the rail 0.4 studs past y + 3). Returns
the placed copy's Handle.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72036. Not derived from any
model file."""

PARTS = ["15068", "99780", "3460", "3666", "3710", "3023b"]

RAIL = Rot(y=90, z=90)  # a plate on its edge, its studs to +y, long along x
RAILS = {8: "3460", 6: "3666", 4: "3710", 2: "3023b"}  # plates 1 x n
LIP = 0.2  # how far a bracket's side face stands past its plate, in studs


def _curb(m, length, base, stripes, rail):
    """The curb, its slopes falling to -y, over (0, 0) to (length, 3) and the rail behind it."""
    with m.step("Base"):
        m.fill(Plate, base, at=(0, 0, 0), size=(length, 3))
    with m.step("Stripes"):
        for k in range(length // 2):
            m.add("15068", stripes[k % len(stripes)], at=(2 * k, 0, 1), rot=3)
    with m.step("Brackets"):
        for k in range(length // 2):
            m.add("99780", rail, at=(2 * k, 2, 1), rot=1)
    with m.step("Rail"):
        at, left = 0, length
        for n in sorted(RAILS, reverse=True):
            while left >= n:
                m.add(RAILS[n], rail, at=(at, 3 + LIP, 1), rot=RAIL)
                at += n
                left -= n


def build(
    m,
    x,
    y,
    z,
    length=8,
    facing=3,
    base="Light Bluish Gray",
    stripes=("Red", "White"),
    rail="Blue",
    title="Curb",
):
    """The curb; returns the placed copy's Handle."""
    if length % 2 or not 2 <= length <= 16:
        raise ValueError(
            f"curb_rumble_strip takes an even length from 2 to 16, not {length}"
        )
    sub = m.submodel(
        title, _curb, length=length, base=base, stripes=tuple(stripes), rail=rail
    )
    # The box's corner after turning the curb about the middle of its strip; its own slopes fall to -y.
    turn = (facing - 3) % 4
    (x0, y0, z0), (x1, y1, _) = sub.box
    cx, cy = length / 2, 1.5
    corners = [(px - cx, py - cy) for px in (x0, x1) for py in (y0, y1)]
    for _ in range(turn):
        corners = [(-py, px) for px, py in corners]
    lo = (min(c[0] for c in corners), min(c[1] for c in corners))
    mx, my = (cx, cy) if turn % 2 == 0 else (cy, cx)
    with m.step(title):
        return m.place(sub, at=(x + mx + lo[0], y + my + lo[1], z + z0), rot=turn)


def example(m):
    """Two curbs at the edges of a stretch of track."""
    with m.step("Track"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(10, 10))
    m.use("curb_rumble_strip", x=1, y=0, z=1, length=8, facing=1)
    m.use("curb_rumble_strip", x=1, y=7, z=1, length=8, title="Far curb", facing=3)
