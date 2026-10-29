# SPDX-License-Identifier: Apache-2.0
"""Window hood: a small peaked hood over a window, two cheese slopes meeting in the middle on a plate that stands out.

m.use("window_hood", x, y, z, width=2, color="White", title="Hood") lays a plate 2 deep and `width` + 2 long over a
window `width` studs wide whose front-left corner is (x, y), at the window's top z, its front row a stud out from the
wall, and on that row two 1 x 1 cheese slopes facing away from each other over the window's middle, with tiles out to
the ends. Returns the z of the plate's top (where the wall goes on).
"""

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350, set 43217. Not derived
from any model file."""

PARTS = [
    "3020",
    "3795",
    "54200",
    "3070b",
    "3069b",
    "60592",
    "60601",
    "3024",
    "3004",
    "3622",
    "3005",
]

PLATES = {4: "3020", 6: "3795"}


def build(m, x, y, z, width=2, color="White", title="Hood"):
    """The plate and its peak; returns the z of the plate's top."""
    n = width + 2
    if n not in PLATES:
        raise ValueError(f"window_hood takes a window 2 or 4 wide, not {width}")
    mid = x + width / 2  # the window's middle
    with m.step(title):
        m.add(PLATES[n], color, at=(x - 1, y - 1, z), rot=1)
        m.add("54200", color, at=(mid - 1, y - 1, z + 1), rot=2)
        m.add("54200", color, at=(mid, y - 1, z + 1), rot=0)
        for a in range(int(x - 1), int(mid - 1)):
            m.add("3070b", color, at=(a, y - 1, z + 1))
        for a in range(int(mid + 1), int(x + width + 1)):
            m.add("3070b", color, at=(a, y - 1, z + 1))
    return z + 1


def example(m):
    """A window with its hood, in a short wall."""
    from emmet_dsl import Brick

    with m.step("Wall and window"):
        m.row(Brick, "Sand Blue", at=(0, 0, 0), length=4)
        f = m.add("60592", "White", at=(1, 0, 3), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601",
            "Trans-Clear",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
        )
        for r in range(2):
            m.add("3005", "Sand Blue", at=(0, 0, 3 + 3 * r))
            m.add("3005", "Sand Blue", at=(3, 0, 3 + 3 * r))
    m.use("window_hood", x=1, y=0, z=9)
