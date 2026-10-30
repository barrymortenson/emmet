# SPDX-License-Identifier: Apache-2.0
"""Mini dormer: a tiny dormer that stands on a studs-up roof, a window 1 x 2 x 2 on a plate with two slope plates
leaning together over it as its gable.

m.use("mini_dormer", x, y, z, facing="-y", color="Yellow", roof="Dark Bluish Gray", title="Dormer") builds, on the
studs at height z, a dormer whose plate covers x to x + 2 and y to y + 1 (facing "-y" or "+y"), or x to x + 1 and y
to y + 2 (facing "-x" or "+x"): a plate 1 x 2, a window 1 x 2 x 2 (60592) with its glass, and two slope plates 45
2 x 1 (15672) on the window's two top studs, sloping down to each side and overhanging it a stud each way. It needs
nothing cut from the roof: put it on any studs-up roof (plate_shingle_roof's shingles, a flat roof's plates), or in a
panel's own function to tilt with it. Returns the z of the gable's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43217. Not derived from any
model file."""

PARTS = ["3023b", "60592", "60601", "15672"]

# The quarter turn that slopes a slope plate down toward each side (its high end the other way).
UP = {"+x": 2, "+y": 3, "-x": 0, "-y": 1}
# The window's quarter turn for each facing, and the axis its 2 studs run along.
WINDOW = {"-y": (1, "x"), "+y": (3, "x"), "-x": (2, "y"), "+x": (0, "y")}


def build(
    m, x, y, z, facing="-y", color="Yellow", roof="Dark Bluish Gray", title="Dormer"
):
    """The plate, the window, and the gable; returns the z of the gable's top."""
    if facing not in WINDOW:
        raise ValueError(f'facing is "-y", "+y", "-x" or "+x", not {facing!r}')
    turn, along = WINDOW[facing]
    with m.step(f"{title}: window"):
        m.add("3023b", color, at=(x, y, z), rot=1 if along == "x" else 0)
        f = m.add("60592", color, at=(x, y, z + 1), rot=turn)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        if along == "x":
            glass = ((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875)
        else:
            glass = ((x0 + x1 - 0.2) / 2, (y0 + y1 - 1.65) / 2, z0 + 0.875)
        m.add("60601", "Trans-Clear", at=glass, rot=turn)
    with m.step(f"{title}: gable"):
        if along == "x":
            m.add("15672", roof, at=(x - 1, y, z + 7), rot=UP["-x"])
            m.add("15672", roof, at=(x + 1, y, z + 7), rot=UP["+x"])
        else:
            m.add("15672", roof, at=(x, y - 1, z + 7), rot=UP["-y"])
            m.add("15672", roof, at=(x, y + 1, z + 7), rot=UP["+y"])
    return z + 10


def example(m):
    """Two dormers on a studs-up plate roof, one facing front and one facing right."""
    from emmet_dsl import Plate

    with m.step("Roof plates"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(8, 6))
    m.use("mini_dormer", x=2, y=1, z=1)
    m.use("mini_dormer", x=6, y=3, z=1, facing="+x", title="Second dormer")
