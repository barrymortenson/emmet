# SPDX-License-Identifier: Apache-2.0
"""Window frame: a window with its glass, built as a sub-assembly and set into a straight brick wall.

m.use("window_frame", x, y, z, length, rows, offset, sill=1, window="1x2x3", facing="front", color="Tan",
frame_color="White", glass_color="Trans-Clear", title="Wall") builds a wall one stud thick and `length` studs long
from (x, y), `rows` bricks tall from z (on the ground or a wall below), with a window `offset` studs from the wall's
start, its bottom `sill` rows up. window is "1x2x2" (2 studs wide, 2 bricks tall) or "1x2x3" (2 wide, 3 tall). facing is the side the window looks out of, which also sets the wall's direction: "front"
(−y) and "back" (+y) walls run along x, "left" (−x) and "right" (+x) ones along y. The bricks go around the window in
a running bond, and the row above it bridges its top. The window is a submodel (the frame, then its glass), placed
in the step of the row it stands on. It brings its own steps, a row at a time, titled "<title>, row 1" and so on, and
returns the z of the wall's top.
"""

from emmet_dsl import Brick

PARTS = [
    "60592",
    "60601",
    "60593",
    "60602",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "2465",
]

# Each window: its frame and glass, its width in studs and height in bricks, and where its glass goes: the glass's
# footprint corner from the frame's, in a frame placed at rot=0 (its front toward +x). The LDraw glass shares the
# frame's origin; these are those offsets in studs and plates.
WINDOWS = {
    "1x2x2": ("60592", "60601", 2, 2, (0.65, 0.175, 0.875)),
    "1x2x3": ("60593", "60602", 2, 3, (0.65, 0.175, 0.875)),
}
# Not yet: "1x4x3": ("60594", "86210", 4, 3, (0.125, 0.15, 1.0)). The glass sits right, but the validator takes
# 60594 ("Window 1 x 4 x 3 without Shutter Tabs") for a shutter, not a frame, so its glass reads as floating.
# The way a wall runs and the quarter turn that faces a window or door out of it, by the side it looks out of.
FACING = {"front": ("x", 3), "back": ("x", 1), "left": ("y", 2), "right": ("y", 0)}


def _window(m, window, frame_color, glass_color):
    frame, glass, _, _, at = WINDOWS[window]
    with m.step("Frame"):
        m.add(frame, frame_color, at=(0, 0, 0), id="frame")
    with m.step("Glass"):
        m.add(glass, glass_color, at=at, id="glass")


def wall_with(m, x, y, z, length, rows, facing, color, title, opening, unit):
    """A straight wall a row at a time, with `unit` (a submodel) set into `opening`: (offset along the wall, the row
    it stands on, its width in studs, its height in rows). The unit goes in at its first row, before that row's
    bricks; the window_frame and door_frame techniques share this. Returns the z of the wall's top."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    axis, turn = FACING[facing]
    offset, sill, width, height = opening
    if offset < 0 or offset + width > length or sill < 0 or sill + height > rows:
        raise ValueError(
            f"a {width}-wide, {height}-row opening at offset {offset}, row {sill} doesn't fit a wall "
            f"{length} long and {rows} rows tall"
        )

    def cell(a):
        return (x + a, y) if axis == "x" else (x, y + a)

    for r in range(rows):
        zr = z + 3 * r
        course = round(
            zr / 3
        )  # the course from the ground sets the stagger, as m.wall does
        if sill <= r < sill + height:
            runs = [(0, offset), (offset + width, length - offset - width)]
        else:
            runs = [(0, length)]
        with m.step(f"{title}, row {r + 1}"):
            if r == sill:
                m.place(unit, at=(*cell(offset), zr), rot=turn)
            for start, n in runs:
                if n <= 0:
                    continue
                # A run of 1 or 2 between the end and the opening has joints at both ends already.
                first = (
                    max(1, Brick.longest(n, 1) // 2) if course % 2 and n > 2 else None
                )
                m.row(
                    Brick,
                    color,
                    at=(*cell(start), zr),
                    length=n,
                    axis=axis,
                    first=first,
                )
    return z + 3 * rows


def build(
    m,
    x,
    y,
    z,
    length,
    rows,
    offset,
    sill=1,
    window="1x2x3",
    facing="front",
    color="Tan",
    frame_color="White",
    glass_color="Trans-Clear",
    title="Wall",
):
    """The wall and its window; returns the z of the wall's top."""
    if window not in WINDOWS:
        raise ValueError(f"window={window!r} isn't a window: use {', '.join(WINDOWS)}")
    _, _, width, height, _ = WINDOWS[window]
    unit = m.submodel(
        f"Window {window}",
        _window,
        window=window,
        frame_color=frame_color,
        glass_color=glass_color,
    )
    return wall_with(
        m,
        x,
        y,
        z,
        length,
        rows,
        facing,
        color,
        title,
        (offset, sill, width, height),
        unit,
    )


def example(m):
    """A wall with a 1 x 2 x 3 window (light blue glass, to show it), standing on the ground."""
    glass = "Trans-Light Blue"
    m.use("window_frame", x=0, y=0, z=0, length=6, rows=5, offset=2, glass_color=glass)
