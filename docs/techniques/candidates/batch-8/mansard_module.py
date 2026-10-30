# SPDX-License-Identifier: Apache-2.0
"""Mansard module: a steep mansard roof face built from 2-stud modules, each two slopes 75 over a bracket that
carries a small round window at the eave.

m.use("mansard_module", x, y, z, modules=3, facing="-y", color="Sand Blue", trim="White", title="Mansard") builds,
on the studs at height z, a run of `modules` modules 2 studs wide on a strip of plates 2 deep that ties them: along x
from (x, y) facing "-y" or "+y" (x to x + 2 x modules, y to y + 2), or along y facing "-x" or "+x". A module: a
bracket 1 x 2 - 2 x 2 (99207) in `trim` at the face, its upright plate's studs facing out, beside a brick and plates
that bring the back row level with it; a plate 2 x 2 across them; two slopes 75 2 x 1 x 3 (4460b) side by side,
their steep faces out; and on the bracket's studs a round tile 2 x 2 with a hole (15535) in `trim`, the window. The
run is built apart, as a submodel, and placed in one step. Put runs on the four sides of a flat roof, the corners
left to a corner piece or a chimney. Returns the z of the slopes' tops (their studs, at the strip's back row).
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = [
    "99207",
    "3004",
    "3023b",
    "3022",
    "4460b",
    "15535",
    "3020",
    "3795",
    "3032",
    "3034",
]

WINDOW = Rot(x=90)  # a tile on its back, its face to -y
# The quarter turn of the run (built facing -y) for each facing, and how far its box reaches past the strip there.
TURN = {"-y": 0, "+x": 1, "+y": 2, "-x": 3}


def _run(m, modules, color, trim):
    """The run in its own frame, facing -y: a strip of plates y 0 to 2, and the modules on it."""
    with m.step("Strip"):
        m.row(Plate, color, at=(0, 0, 0), length=2 * modules, width=2)
    for k in range(modules):
        x = 2 * k
        with m.step(f"Module {k + 1}: bracket"):
            # rot 3 turns the upright face (its studs) to -y, just in front of the strip.
            m.add("99207", trim, at=(x, -0.2, 1), rot=3)
            m.add("3004", color, at=(x, 1, 1), rot=1)
            m.add("3004", color, at=(x, 0, 2), rot=1)
            m.add("3023b", color, at=(x, 1, 4), rot=1)
            m.add("3023b", color, at=(x, 0, 5), rot=1)
            m.add("3023b", color, at=(x, 1, 5), rot=1)
        with m.step(f"Module {k + 1}: slopes"):
            m.add("3022", color, at=(x, 0, 6))
            m.add("4460b", color, at=(x, 0, 7), rot=3)
            m.add("4460b", color, at=(x + 1, 0, 7), rot=3)
        with m.step(f"Module {k + 1}: window"):
            m.add("15535", trim, at=(x, -0.6, 1), rot=WINDOW)


def build(
    m, x, y, z, modules=3, facing="-y", color="Sand Blue", trim="White", title="Mansard"
):
    """The run, built apart and placed; returns the z of the slopes' tops."""
    if facing not in TURN:
        raise ValueError(f'facing is "-y", "+y", "-x" or "+x", not {facing!r}')
    sub = m.submodel(f"{title}: run", _run, modules=modules, color=color, trim=trim)
    # The run's box reaches 0.6 of a stud out past the strip, on the side it faces.
    at = {
        "-y": (x, y - 0.6, z),
        "+y": (x, y, z),
        "+x": (x, y, z),
        "-x": (x - 0.6, y, z),
    }[facing]
    with m.step(title):
        m.place(sub, at=at, rot=TURN[facing])
    return z + 16


def example(m):
    """A flat-topped block with mansard runs on its front and back."""
    from emmet_dsl import Plate

    with m.step("Deck"):
        m.fill(Plate, "Light Bluish Gray", at=(0, 0, 0), size=(6, 6))
    m.use("mansard_module", x=0, y=0, z=1, modules=3)
    m.use(
        "mansard_module", x=0, y=4, z=1, modules=3, facing="+y", title="Mansard, back"
    )
