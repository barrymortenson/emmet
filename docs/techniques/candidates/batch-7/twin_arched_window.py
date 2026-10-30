# SPDX-License-Identifier: Apache-2.0
"""Twin arched window: two round-topped windows side by side under one curved arch, a slim round column in front of the
seam between them.

m.use("twin_arched_window", x, y, z, color="Tan", frame="White", sill="Dark Orange", title="Twin window") sets, in a
wall one stud thick along x facing front (-y), a sill Plate 2 x 6 from (x - 1, y - 1) at z, its back row in the wall and
its front row a stud out; on it two Windows 1 x 2 x 2 2/3 with Rounded Top and a lattice pane (30044, 30046) at x and x
+ 2, a jamb of a brick and a plate each side at x - 1 and x + 4, and an Arch 1 x 6 x 2 with Curved Top (6183) on the
jambs at z + 5, so the windows' round tops fill the corners of its opening. On the sill's front row a jumper centers a
column of round 1 x 1 pieces (a round plate, two round bricks, a round plate) in front of the windows' seam. Each window
(frame, then pane) is a submodel. It takes the wall's cells x - 1 to x + 4, from z to z + 11; the sill is a plate course
in the wall. Returns the z of the arch's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["30044", "30046", "6183", "3795", "15573", "6141", "3062b", "3005", "3024"]

# The pane's footprint corner from its frame's, in a frame placed at rot=0: LDraw puts its hinge 24 LDU down and
# 4 LDU behind the frame's origin.
PANE_AT = (0.2, 0.162, 0.906)


def _window(m, frame):
    with m.step("Frame"):
        m.add("30044", frame, at=(0, 0, 0), id="frame")
    with m.step("Pane"):
        m.add("30046", frame, at=PANE_AT, id="pane")


def build(
    m, x, y, z, color="Tan", frame="White", sill="Dark Orange", title="Twin window"
):
    """The sill, the windows and their jambs, the column, and the arch; returns the z of the arch's top."""
    unit = m.submodel("Arched window", _window, frame=frame)
    with m.step(f"{title}: sill and windows"):
        m.add("3795", sill, at=(x - 1, y - 1, z), rot=1)
        m.place(unit, at=(x, y, z + 1), rot=3)
        m.place(unit, at=(x + 2, y, z + 1), rot=3)
    with m.step(f"{title}: jambs"):
        for cx in (x - 1, x + 4):
            m.add("3005", color, at=(cx, y, z + 1))
            m.add("3024", color, at=(cx, y, z + 4))
    with m.step(f"{title}: column"):
        m.add("15573", color, at=(x + 1, y - 1, z + 1), rot=1)
        m.add("6141", color, at=(x + 1.5, y - 1, z + 2))
        m.add("3062b", color, at=(x + 1.5, y - 1, z + 3))
        m.add("3062b", color, at=(x + 1.5, y - 1, z + 6))
        m.add("6141", color, at=(x + 1.5, y - 1, z + 9))
    with m.step(f"{title}: arch"):
        m.add("6183", color, at=(x - 1, y, z + 5), rot=1)
    return z + 11


def example(m):
    """The window in a short wall on a course of bricks."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Tan", at=(-2, 0, 0), length=8)
    z = m.use("twin_arched_window", x=0, y=0, z=3)
    with m.step("Wall beside the sill"):
        for cx in (-2, 5):
            m.add("3024", "Tan", at=(cx, 0, 3))
    for r in range(3):
        with m.step(f"Wall beside the window, row {r + 1}"):
            for cx in (-2, 5):
                m.add("3005", "Tan", at=(cx, 0, 4 + 3 * r))
    with m.step("Wall beside the arch"):
        for cx in (-2, 5):
            m.add("3024", "Tan", at=(cx, 0, 13))
    with m.step("Wall over the arch"):
        m.row(Brick, "Tan", at=(-2, 0, z), length=8)
