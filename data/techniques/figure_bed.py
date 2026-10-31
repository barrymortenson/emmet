# SPDX-License-Identifier: Apache-2.0
"""Figure bed: a single bed for a figure, 2 x 6 studs with a headboard or 2 x 8 with four posts, a pillow, and a
checkered quilt, built apart and placed.

m.use("figure_bed", x, y, z, facing="-y", posts=False, frame="Reddish Brown", quilt="Blue", quilt2="White",
pillow="White", title="Bed", id=None) stands a bed on the studs at height z, its footprint's min corner at (x, y): 2 x 6
(2 x 8 with posts) with its foot toward `facing` "-y" or "+y", turned for "+x" or "-x", so its head goes against the
wall behind. It's two 1 x 2 plates as legs under a plate frame, a curved slope 2 x 2 as the pillow, and a quilt of 1 x
1 tiles checkered in quilt and quilt2 that a figure lies on (its top 3 plates up). posts=False gives a brick and tile
headboard; posts=True gives four corner posts of round bricks with cones on top, tall at the head and short at the
foot (the four-poster), on a frame 2 studs longer, so the foot posts stand past the figure's feet. A figure lies on 5
studs of quilt and pillow (6 in the four-poster), and the bed keeps 2 bricks clear over them
(docs/spec/building-rules.md, rule 3). It's a submodel, built once in its own steps, "<title>, frame", "<title>, head"
(with the pillow), "<title>, foot" (with posts), and "<title>, quilt", placed in a step titled "<title>" with the id
`id` (by default the submodel's own), so that room moves with every copy; it returns the placed copy's Handle.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21335, set 43268, set 11377.
Not derived from any model file."""

PARTS = ["3795", "3034", "3023b", "15068", "3070b", "3004", "3069b", "3062b", "4589"]

LIE = 6  # the room kept clear over where a figure lies, in plates: its depth, and room to lay it down

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


def _put(m, sub, x, y, z, facing, foot, id=None):
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
    return m.place(
        sub, at=(x - fl[0] + lo[0], y - fl[1] + lo[1], z + bz0), rot=k, id=id
    )


def _bed(m, posts, frame, quilt, quilt2, pillow, title):
    """The bed in its own frame: 2 wide, its foot at y = 0 and its head at y = 6 (y = 8 with posts)."""
    length = 8 if posts else 6
    # The first row a figure lies on: the four-poster's foot posts stand on row 0.
    foot = 1 if posts else 0
    head = length - 1  # the headboard's or the head posts' row
    with m.step(f"{title}, frame"):
        m.add("3023b", frame, at=(0, 0, 0), rot=1, id="legs/foot")
        m.add("3023b", frame, at=(0, head, 0), rot=1, id="legs/head")
        m.add("3034" if posts else "3795", frame, at=(0, 0, 1), id="frame")
    with m.step(f"{title}, head"):
        m.add("15068", pillow, at=(0, head - 2, 2), rot=1, id="pillow")
        if posts:
            for i in range(2):
                m.stack(
                    "3062b", frame, at=(i, head, 2), count=2, id=f"post/head/{i + 1}"
                )
                m.add("4589", frame, at=(i, head, 8), id=f"post/head/{i + 1}/top")
        else:
            m.add("3004", frame, at=(0, head, 2), rot=1, id="headboard")
            m.add("3069b", frame, at=(0, head, 5), rot=1, id="headboard/top")
    if posts:
        with m.step(f"{title}, foot"):
            for i in range(2):
                m.add("3062b", frame, at=(i, 0, 2), id=f"post/foot/{i + 1}")
                m.add("4589", frame, at=(i, 0, 5), id=f"post/foot/{i + 1}/top")
    with m.step(f"{title}, quilt"):
        for ly in range(foot, head - 2):
            for lx in range(2):
                m.add(
                    "3070b",
                    quilt if (lx + ly) % 2 == 0 else quilt2,
                    at=(lx, ly, 2),
                    id=f"quilt/{lx}-{ly}",
                )
    # Over the quilt (its top 3 plates up) and the pillow (4).
    m.keep_clear(
        [
            ((0, foot, 3), (2, head - 2, 3 + LIE)),
            ((0, head - 2, 4), (2, head, 3 + LIE)),
        ],
        "a figure lying in the bed",
    )


def build(
    m,
    x,
    y,
    z,
    facing="-y",
    posts=False,
    frame="Reddish Brown",
    quilt="Blue",
    quilt2="White",
    pillow="White",
    title="Bed",
    id=None,
):
    """The bed; returns the placed copy's Handle."""
    sub = m.submodel(
        "bed",
        _bed,
        posts=posts,
        frame=frame,
        quilt=quilt,
        quilt2=quilt2,
        pillow=pillow,
        title=title,
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, 2, 8 if posts else 6), id=id)


def example(m):
    """A plain bed and a four-poster side by side, their heads to the back."""
    with m.step("Floor"):
        m.add("3036", "Tan", at=(0, 0, 0))
    m.use("figure_bed", x=0, y=2, z=1)
    m.use(
        "figure_bed",
        x=3,
        y=0,
        z=1,
        posts=True,
        quilt="Dark Green",
        quilt2="Tan",
        title="Four-poster",
    )
