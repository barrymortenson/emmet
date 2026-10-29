# SPDX-License-Identifier: Apache-2.0
"""Control console: a desk of controls for a cockpit or a bridge, a course of 45° slopes as the sloping panel with
buttons, grilles, and levers on the studs along its top.

m.use("control_console", x, y, z, length=4, rows=1, color="Dark Bluish Gray", panel="Black", seed=1, title="Console")
builds, on the 2 × `length` studs whose front-left corner is (x, y), `rows` courses of bricks 2 wide from z, then a
course of slopes 45 2 x 1 (3040b) facing front (-y), in `panel`, the operator's screens. Each slope leaves one stud at
the back; on that row go the controls, chosen from `seed` (the same seed gives the same console): round tiles 1 x 1
(98138) in transparent colors as buttons, a control stick (4592 and 4593) leaning toward the operator as a lever, and
grille tiles 1 x 2 (2412b) across two studs. length is 2 to 12. It brings its own steps and returns the z of the
slopes' top (z + 3 × rows + 3).
"""

import random

from emmet_dsl import Brick, frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = [
    "3001",
    "3002",
    "3003",
    "3004",
    "2456",
    "3007",
    "3006",
    "3040b",
    "98138",
    "4592",
    "4593",
    "2412b",
]

BUTTONS = ["Trans-Red", "Trans-Yellow", "Trans-Green", "Trans-Light Blue"]
LEAN = 25  # how far a lever leans toward the operator, in degrees


def _stick(m, color, cell, z):
    """A control stick on the stud at cell (x, y) at height z: its base turned so the pivot runs along x, and the
    stick leaning toward the operator (-y)."""
    base = m.add(
        "4592", "Light Bluish Gray", at=(cell[0] + 0.1, cell[1] + 0.1, z), rot=1
    )
    # The stick turns on the base's pivot, 2 LDU above the base's origin, which is a plate up from its underside.
    pivot = (base.min[0] + 0.4, base.min[1] + 0.4, z + 1 + 2 / frame.PLATE)
    turn = frame.rotation(0, 0, 90)
    lean = frame.rotation(LEAN, 0, 0)
    r_dsl = tuple(
        tuple(sum(lean[i][k] * turn[k][j] for k in range(3)) for j in range(3))
        for i in range(3)
    )
    r = frame.to_ldraw_rotation(r_dsl)
    p = frame.to_ldraw_point(pivot)
    m.add_raw("4593", color, [*p, *r[0], *r[1], *r[2]])


def build(
    m,
    x,
    y,
    z,
    length=4,
    rows=1,
    color="Dark Bluish Gray",
    panel="Black",
    seed=1,
    title="Console",
):
    """The desk, the sloping panel, and the controls; returns the z of the slopes' top."""
    if not 2 <= length <= 12 or rows < 0:
        raise ValueError(
            f"control_console takes a length of 2 to 12 and rows of 0 or more, not {length}, {rows}"
        )
    for r in range(rows):
        with m.step(f"{title}: desk, row {r + 1}"):
            m.row(Brick, color, at=(x, y, z + 3 * r), length=length, width=2)
    top = z + 3 * rows
    for start in range(0, length, 8):
        part = f" ({start // 8 + 1})" if length > 8 else ""
        with m.step(f"{title}: panel{part}"):
            for i in range(start, min(start + 8, length)):
                m.add("3040b", panel, at=(x + i, y, top), rot=3)
    rng = random.Random(seed)
    controls, i = [], 0
    while i < length:
        pick = rng.random()
        if i + 1 < length and pick < 0.3:
            controls.append(("grille", i))
            i += 2
        elif pick < 0.55:
            controls.append(("lever", i))
            i += 1
        else:
            controls.append(("button", i))
            i += 1
    # In steps of at most 8 parts: a lever is two (its base and its stick).
    groups, size = [[]], 0
    for c in controls:
        n = 2 if c[0] == "lever" else 1
        if size + n > 8:
            groups.append([])
            size = 0
        groups[-1].append(c)
        size += n
    for g, group in enumerate(groups):
        part = f" ({g + 1})" if len(groups) > 1 else ""
        with m.step(f"{title}: controls{part}"):
            for kind, i in group:
                cell = (x + i, y + 1)
                if kind == "grille":
                    m.add("2412b", "Light Bluish Gray", at=(*cell, top + 3), rot=1)
                elif kind == "button":
                    m.add("98138", rng.choice(BUTTONS), at=(*cell, top + 3))
                else:
                    _stick(m, "Black", cell, top + 3)
    return top + 3


def example(m):
    """A console 6 studs long on a deck."""
    with m.step("Deck"):
        m.add("3036", "Light Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("control_console", x=1, y=2, z=1, length=6, seed=3)
