# SPDX-License-Identifier: Apache-2.0
"""Door frame: a door in its frame, built as a sub-assembly and set into a straight brick wall.

m.use("door_frame", x, y, z, length, rows, offset, sill=0, facing="front", color="Tan", frame_color="White",
door_color="Reddish Brown", title="Wall") builds a wall one stud thick and `length` studs long from (x, y), `rows`
bricks tall from z (at least sill + 6), with a door 1 x 4 x 6 (a frame 4 studs wide and 6 bricks tall, and its door, closed)
`offset` studs from the wall's start, its bottom `sill` rows up: 0 on the ground floor. facing is the side the door
opens out of, which also sets the wall's direction: "front" (−y) and "back" (+y) walls run along x, "left" (−x) and
"right" (+x) ones along y. The bricks go around the door in a running bond, and the row above it, if any, bridges its
top. The door in its frame is a submodel (the frame, then the door), placed in the step of its own bottom row (row sill + 1). It
brings its own steps, a row at a time, titled "<title>, row 1" and so on, and returns the z of the wall's top.
"""

from emmet_dsl.techniques.window_frame import wall_with

PARTS = [
    "60596",
    "60623",
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

# The door's footprint corner from its frame's, in a frame placed at rot=0 (its front toward +x): the door's hinge
# pins go in the frame's holes, 32 LDU left of its middle and 5 LDU behind it.
DOOR_AT = (0.1, 0.2, 0.875)


def _door(m, frame_color, door_color):
    with m.step("Frame"):
        m.add("60596", frame_color, at=(0, 0, 0), id="frame")
    with m.step("Door"):
        m.add("60623", door_color, at=DOOR_AT, id="door")


def build(
    m,
    x,
    y,
    z,
    length,
    rows,
    offset,
    sill=0,
    facing="front",
    color="Tan",
    frame_color="White",
    door_color="Reddish Brown",
    title="Wall",
):
    """The wall and its door; returns the z of the wall's top."""
    unit = m.submodel("Door", _door, frame_color=frame_color, door_color=door_color)
    return wall_with(
        m, x, y, z, length, rows, facing, color, title, (offset, sill, 4, 6), unit
    )


def example(m):
    """A wall with a door, standing on the ground."""
    m.use("door_frame", x=0, y=0, z=0, length=8, rows=7, offset=2)
