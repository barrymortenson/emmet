# SPDX-License-Identifier: Apache-2.0
"""Interior arch doorway: a partition wall one brick thick between two rooms, with a doorway 4 studs wide closed at the
top by a raised arch in a contrasting color.

m.use("interior_arch_doorway", x, y, z, length=8, axis="x", gap=2, opening=5, rows=7, color="White", arch="Tan",
title="Partition") builds a straight wall `length` studs long along `axis` ("x" or "y") from the cell (x, y), `rows`
bricks tall, standing on the studs at height z. The doorway is 4 studs wide, `gap` studs from the wall's start, open
from the floor through `opening` courses (5 is 15 plates, room for a figure with a hat); on the next course an Arch 1 x
6 Raised spans it with a leg on the wall each side, and the wall goes on above it in running bond. gap is at least 1 and
gap + 5 at most length, so each leg has wall under it. It's a submodel, built once in its own steps, a row at a time,
"<title>, row 1" and so on and "<title>, arch", placed in a step titled "<title>"; it returns the placed copy's Handle.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21335. Not derived from any
model file."""

PARTS = ["92950", "3008", "3010", "3004", "3005"]

FACING = {
    "-y": 0,
    "+x": 1,
    "+y": 2,
    "-x": 3,
}  # quarter turns that point a front built toward -y the given way


def _turn(px, py, k):
    for _ in range(k):
        px, py = -py, px
    return px, py


def _put(m, sub, x, y, z, facing, foot):
    """Places sub so the rectangle foot = (x0, y0, x1, y1) of its own frame lands with its min corner at (x, y) and
    its own z = 0 at z, its front (built toward -y) toward `facing`."""
    if facing not in FACING:
        raise ValueError(f"facing must be one of {', '.join(FACING)}, not {facing!r}")
    k = FACING[facing]
    (bx0, by0, bz0), (bx1, by1, _) = sub.box
    box = [_turn(px, py, k) for px in (bx0, bx1) for py in (by0, by1)]
    feet = [_turn(px, py, k) for px in (foot[0], foot[2]) for py in (foot[1], foot[3])]
    lo = (min(p[0] for p in box), min(p[1] for p in box))
    fl = (min(p[0] for p in feet), min(p[1] for p in feet))
    return m.place(sub, at=(x - fl[0] + lo[0], y - fl[1] + lo[1], z + bz0), rot=k)


def _partition(m, length, gap, opening, rows, color, arch, title):
    """The wall in its own frame: along x from (0, 0), the doorway from x = gap to gap + 4."""
    door = [(gap + i, 0) for i in range(4)]
    span = [(gap - 1 + i, 0) for i in range(6)]
    for r in range(rows):
        with m.step(f"{title}, row {r + 1}"):
            if r < opening:
                m.wall(0, 0, 3 * r, length=length, rows=1, color=color, openings=door)
            elif r == opening:
                m.wall(0, 0, 3 * r, length=length, rows=1, color=color, openings=span)
            else:
                m.wall(0, 0, 3 * r, length=length, rows=1, color=color)
        if r == opening:
            with m.step(f"{title}, arch"):
                m.add("92950", arch, at=(gap - 1, 0, 3 * r), rot=1, id="arch")


def build(
    m,
    x,
    y,
    z,
    length=8,
    axis="x",
    gap=2,
    opening=5,
    rows=7,
    color="White",
    arch="Tan",
    title="Partition",
):
    """The partition; returns the placed copy's Handle."""
    if axis not in ("x", "y"):
        raise ValueError(f'axis must be "x" or "y", not {axis!r}')
    if gap < 1 or gap + 5 > length:
        raise ValueError(
            f"the doorway needs wall each side: gap from 1 to {length - 5}, not {gap}"
        )
    if not 0 < opening < rows:
        raise ValueError(f"opening is 1 to {rows - 1} courses, not {opening}")
    sub = m.submodel(
        "partition",
        _partition,
        length=length,
        gap=gap,
        opening=opening,
        rows=rows,
        color=color,
        arch=arch,
        title=title,
    )
    with m.step(title):
        return _put(m, sub, x, y, z, "-y" if axis == "x" else "+x", (0, 0, length, 1))


def example(m):
    """A partition across a floor, its arch over a doorway between the two halves."""
    with m.step("Floor"):
        m.add("3036", "Reddish Brown", at=(0, 0, 0))
    m.use("interior_arch_doorway", x=3, y=0, z=1, axis="y")
