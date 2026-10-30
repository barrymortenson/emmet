# SPDX-License-Identifier: Apache-2.0
"""Tower crown: the top that makes a micro tower a tower, rounded, pyramid, or gabled, in the shaft's own color.

m.use("tower_crown", x, y, z, width=2, depth=1, kind="rounded", color="Light Bluish Gray", title="Crown") tops the width
x depth tower whose front-left corner is (x, y), on its studs at height z. kind "rounded": bricks with curved tops
(37352 over two studs along x, 49307 over one), and on a tower 3 wide a 1 x 1 brick lifts the first one, so the crown
has two peaks at different heights. kind "pyramid": a point, a 1 x 1 pyramid slope (22388) on a 1 x 1 tower, a 2 x 2 x 2
quadruple slope (3688) on a 2 x 2 one, and twin points on a 1 x 2 one. kind "gable": a 45 degree double slope, 3043 on a
2 x 2 tower with its ridge along x, 3044c on a 1 x 2 one with its ridge across it, or a single cheese slope, facing
front, on a 1 x 1. width and depth are 1 or 2 (rounded also takes a width of 3). Returns the z of the crown's highest
point.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = ["37352", "49307", "3005", "22388", "3688", "54200", "3043", "3044c"]

KINDS = ("rounded", "pyramid", "gable")


def build(
    m,
    x,
    y,
    z,
    width=2,
    depth=1,
    kind="rounded",
    color="Light Bluish Gray",
    title="Crown",
):
    """The crown; returns the z of its top."""
    if kind not in KINDS:
        raise ValueError(
            f"tower_crown: kind is one of {', '.join(KINDS)}, not {kind!r}"
        )
    widest = 3 if kind == "rounded" else 2
    if not (1 <= width <= widest and 1 <= depth <= 2):
        raise ValueError(
            f"tower_crown {kind} takes a width of 1 to {widest} and a depth of 1 or 2, not {width}, {depth}"
        )
    top = z
    with m.step(title):
        if kind == "rounded":
            for j in range(depth):
                i = 0
                if width == 3:
                    # The tall peak: a 1 x 1 brick under a curved top, on the back row a brick higher still.
                    lift = 3 * (1 + j)
                    for k in range(1 + j):
                        m.add("3005", color, at=(x, y + j, z + 3 * k))
                    m.add("49307", color, at=(x, y + j, z + lift))
                    top = max(top, z + lift + 2)
                    i = 1
                while i < width:
                    if i + 1 < width:
                        m.add("37352", color, at=(x + i, y + j, z), rot=1)
                        top = max(top, z + 3)
                        i += 2
                    else:
                        m.add("49307", color, at=(x + i, y + j, z))
                        top = max(top, z + 2)
                        i += 1
        elif kind == "pyramid":
            if (width, depth) == (1, 1):
                m.add("22388", color, at=(x, y, z))
                top = z + 2
            elif (width, depth) == (2, 2):
                m.add("3688", color, at=(x, y, z))
                top = z + 6
            else:
                # Twin points, one on each stud.
                for k in range(2):
                    m.add(
                        "22388",
                        color,
                        at=(x + k * (width == 2), y + k * (depth == 2), z),
                    )
                top = z + 2
        else:
            if (width, depth) == (1, 1):
                m.add("54200", color, at=(x, y, z), rot=3)
                top = z + 2
            elif (width, depth) == (2, 2):
                m.add("3043", color, at=(x, y, z), rot=1)
                top = z + 3
            else:
                # A 2 x 1 double slope along the long side: its ridge runs across the short one.
                m.add("3044c", color, at=(x, y, z), rot=0 if width == 2 else 1)
                top = z + 3
    return top


def example(m):
    """Three short towers, one crown of each kind."""
    from emmet_dsl import Brick, Plate

    with m.step("Ground"):
        m.add(Plate(4, 10), "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    with m.step("Left tower"):
        m.stack(Brick(1, 3), "Dark Blue", at=(0, 1, 1), count=4, rot=1)
    with m.step("Middle tower"):
        m.stack(Brick(2, 2), "White", at=(4, 1, 1), count=3)
    with m.step("Right tower"):
        m.stack(Brick(2, 2), "Light Bluish Gray", at=(7, 1, 1), count=5)
    m.use(
        "tower_crown",
        x=0,
        y=1,
        z=13,
        width=3,
        depth=1,
        kind="rounded",
        color="Dark Blue",
        title="Rounded crown",
    )
    m.use(
        "tower_crown",
        x=4,
        y=1,
        z=10,
        width=2,
        depth=2,
        kind="pyramid",
        color="White",
        title="Pyramid crown",
    )
    m.use(
        "tower_crown",
        x=7,
        y=1,
        z=16,
        width=2,
        depth=2,
        kind="gable",
        color="Light Bluish Gray",
        title="Gable crown",
    )
