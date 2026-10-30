# SPDX-License-Identifier: Apache-2.0
"""Posable neck ball: a head on a ball joint at the front of a body, so it turns and nods: a plate with a ball on the
body, a plate with a socket under the back of the head.

m.use("posable_neck_ball", x, y, z, turn=0, nod=0, width=4, depth=4, courses=2, color="White", joint_color="Dark Bluish
Gray", title="Head") puts a plate 1 x 2 with a ball on its end (22890) on the studs of the body's top at z, on the cells
(x, y) and (x, y + 1) of the body's front edge, the ball past the front at (x + 0.5, y - 0.5, z + 0.5). The head, built
apart, is a block `width` studs wide and `depth` deep (each 4 or 6) on a base of plates with a plate 1 x 2 with a socket
(14418) in its back middle, its socket reaching back past the head; a plate across the base ties it, then `courses`
courses of bricks (1 to 4), and plates and tiles on top. The head goes on with its socket on the ball, in front of the
body, turned `turn` degrees (positive to its left, up to 45) and nodded `nod` degrees (positive tips its face up, -30 to
30); the joint's friction holds the pose. The head is a submodel. Returns the head copy's Handle.
"""

from emmet_dsl import Brick, Plate, Rot, Tile, frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21368 and 43279 (the heads
there turn on an axle and a turntable; a ball joint does both). Not derived from any model file."""

PARTS = [
    "22890",
    "14418",
    "3020",
    "3022",
    "3023b",
    "3031",
    "3032",
    "3001",
    "3003",
    "3004",
    "87079",
    "3068b",
]

SOCKET = BALL = (
    30,
    4,
    0,
)  # the socket's and the ball's center on their plates, in each part's own frame (LDU)


def _put(m, part, color, point, local, r_dsl):
    """Places a part turned by r_dsl (DSL axes) so the point `local` of its own frame (LDU) lands on `point` (DSL)."""
    r = frame.to_ldraw_rotation(r_dsl)
    p = frame.to_ldraw_point(point)
    pos = [p[i] - sum(r[i][k] * local[k] for k in range(3)) for i in range(3)]
    return m.add_raw(part, color, [*pos, *r[0], *r[1], *r[2]])


def _at(origin, local, r_dsl):
    """Where `local` (a part's own frame, LDU) lands, in DSL units, for a part whose origin is at `origin` (DSL)."""
    r = frame.to_ldraw_rotation(r_dsl)
    o = frame.to_ldraw_point(origin)
    w = [o[i] + sum(r[i][k] * local[k] for k in range(3)) for i in range(3)]
    return (-w[2] / frame.STUD, w[0] / frame.STUD, -w[1] / frame.PLATE)


def _world(sub, point, turn):
    """Where a point of a submodel (DSL units, its own frame) lands when the submodel is placed at (0, 0, 0)."""
    pos, r, _ = frame.place(
        frame.box_in_ldraw(*sub.box), (0, 0, 0), frame.rotation(turn.x, turn.y, turn.z)
    )
    p = frame.to_ldraw_point(point)
    w = [sum(r[i][k] * p[k] for k in range(3)) + pos[i] for i in range(3)]
    return (-w[2] / frame.STUD, w[0] / frame.STUD, -w[1] / frame.PLATE)


def _head(m, width, depth, courses, color, joint_color):
    """The head facing front (-y), its base's front-left corner at the origin, the socket plate in its back middle."""
    mid = width // 2 - 1  # the socket plate's column
    with m.step("Base, with the socket"):
        # The plate's origin is its top middle: it covers (mid, depth - 2) to (mid + 1, depth), the socket behind.
        _put(
            m,
            "14418",
            joint_color,
            (mid + 0.5, depth - 1, 1),
            (0, 0, 0),
            frame.rotation(),
        )
        m.row(Plate, color, at=(0, 0, 0), length=width, axis="x", width=depth - 2)
        m.row(Plate, color, at=(0, depth - 2, 0), length=mid, axis="x", width=2)
        m.row(
            Plate,
            color,
            at=(mid + 1, depth - 2, 0),
            length=width - mid - 1,
            axis="x",
            width=2,
        )
    with m.step("Tie plate"):
        m.row(Plate, color, at=(0, 0, 1), length=width, axis="x", width=depth)
    for c in range(courses):
        with m.step(f"Course {c + 1}"):
            for j in range(0, depth, 2):
                m.row(
                    Brick,
                    color,
                    at=(0, j, 2 + 3 * c),
                    length=width,
                    axis="x",
                    width=2,
                    first=2 if (c + j // 2) % 2 else None,
                )
    top = 2 + 3 * courses
    with m.step("Top"):
        m.row(Plate, color, at=(0, 0, top), length=width, axis="x", width=depth)
        for j in range(0, depth, 2):
            m.row(Tile, color, at=(0, j, top + 1), length=width, axis="x", width=2)
    return _at((mid + 0.5, depth - 1, 1), SOCKET, frame.rotation())


def build(
    m,
    x,
    y,
    z,
    turn=0,
    nod=0,
    width=4,
    depth=4,
    courses=2,
    color="White",
    joint_color="Dark Bluish Gray",
    title="Head",
):
    """The ball plate on the body, then the head on the ball; returns the head copy's Handle."""
    if width not in (4, 6) or depth not in (4, 6):
        raise ValueError(
            f"posable_neck_ball takes a width and depth of 4 or 6, not {width}, {depth}"
        )
    if not 1 <= courses <= 4:
        raise ValueError(f"posable_neck_ball takes 1 to 4 courses, not {courses}")
    if not -45 <= turn <= 45 or not -30 <= nod <= 30:
        raise ValueError(
            f"posable_neck_ball takes a turn of -45 to 45 and a nod of -30 to 30, not {turn}, {nod}"
        )
    with m.step(f"{title}: ball"):
        # Turned so the ball points to the front: its plate's origin is its top middle, over (x, y) to (x, y + 2).
        ball_turn = frame.rotation(0, 0, 180)
        _put(m, "22890", joint_color, (x + 0.5, y + 1, z + 1), (0, 0, 0), ball_turn)
    ball = _at((x + 0.5, y + 1, z + 1), BALL, ball_turn)
    socket = {}

    def head(m, **kw):
        socket["at"] = _head(m, **kw)

    sub = m.submodel(
        "posable_neck_head",
        head,
        width=width,
        depth=depth,
        courses=courses,
        color=color,
        joint_color=joint_color,
    )
    if (
        "at" not in socket
    ):  # the submodel was built before, for another head of the same size and colors
        mid = width // 2 - 1
        socket["at"] = _at((mid + 0.5, depth - 1, 1), SOCKET, frame.rotation())
    rot = Rot(x=-nod, z=turn)  # a turn about x by a positive angle tips the front down
    landed = _world(sub, socket["at"], rot)
    with m.step(f"{title}: on the ball"):
        return m.place(sub, at=tuple(ball[i] - landed[i] for i in range(3)), rot=rot)


def example(m):
    """A body 4 studs wide on a base, its head turned a little and nodding up."""
    with m.step("Base"):
        m.add("3032", "Dark Bluish Gray", at=(0, -2, 0))  # Plate 4 x 6
    for c in range(2):
        with m.step(f"Body, course {c + 1}"):
            for j in (0, 2):
                m.row(
                    Brick,
                    "White",
                    at=(0, j, 1 + 3 * c),
                    length=4,
                    axis="x",
                    width=2,
                    first=2 if (c + j // 2) % 2 else None,
                )
    with m.step("Body, top"):
        m.add("3031", "White", at=(0, 0, 7))  # Plate 4 x 4
    m.use(
        "posable_neck_ball", x=1, y=0, z=8, turn=20, nod=10, width=4, depth=4, courses=1
    )
