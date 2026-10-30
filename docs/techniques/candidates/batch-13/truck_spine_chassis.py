# SPDX-License-Identifier: Apache-2.0
"""Truck spine chassis: a 6-wide city truck's frame, a 2-wide spine with the axles under it, widened by plates laid
across and locked by a keel brick.

m.use("truck_spine_chassis", x, y, z, length=20, axles=2, color="Black", arch="Red", rim="Light Bluish Gray",
title="Chassis") stands a truck chassis 6 studs wide (y to y + 6) and `length` long on the ground at height z, its
front to -x. A spine of 2-wide plates runs down the middle; under it at each axle two plates with one wheel pin (4488)
carry a 14 mm rim and its tire on each side; over each axle a pair of 4 x 2.5 x 1 mudguards (98282) sit on the spine
and a 1 x 2 plate each side. Between the arches 2 x 6 plates laid across widen the spine to 6, and a 1 x 8 brick down
the middle (the keel, as long as the widest span allows) locks the joint in the spine. axles: 2, or 3 for a tandem
rear axle (the two rear axles 4 studs apart). The front axle is 3 studs from the front, the last 6 from the back.
Returns the z of the top of the plates across, where the cab and the bed go (the keel stands on it).
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 60509, set 60440. Not derived
from any model file."""

PARTS = [
    "4488",
    "11208",
    "11209",
    "98282",
    "3023b",
    "3795",
    "3666",
    "3008",
    "3009",
    "3010",
]

TIRE = 26.25 / 8  # the tire's radius (11209), in plates
OUT_LEFT = (
    0,
    0,
    1,
    0,
    1,
    0,
    -1,
    0,
    0,
)  # a wheel's turn, its hub on a pin that points to -y
OUT_RIGHT = (0, 0, -1, 0, 1, 0, 1, 0, 0)  # and to +y
SPINE = (16, 14, 12, 10, 8, 6, 4)  # Plate 2 x n lengths, longest first


def _raw(m, part, color, c, turn):
    """A part placed by its LDraw origin at the DSL point c (studs, studs, plates)."""
    x, y, z = c
    return m.add_raw(part, color, (20 * y, -8 * z, -20 * x, *turn))


BRICKS = {
    8: "3008",
    6: "3009",
    4: "3010",
    3: "3622",
    2: "3004",
}  # Brick 1 x n, for the keel


def _joint(x, length, span):
    """Where the spine's two plates meet: in the span (start, end) between arches, as near its middle as the plate
    lengths allow."""
    s, e = span
    for j in sorted(range(s + 1, e), key=lambda j: abs(2 * j - s - e)):
        if j - x in SPINE and x + length - j in SPINE:
            return j
    raise ValueError(
        f"truck_spine_chassis can't lay a spine {length} long: try another length"
    )


def build(
    m,
    x,
    y,
    z,
    length=20,
    axles=2,
    color="Black",
    arch="Red",
    rim="Light Bluish Gray",
    title="Chassis",
):
    """The axles, spine, arches, cross plates, and keel; returns the z of the cross plates' top."""
    if axles not in (2, 3):
        raise ValueError(f"truck_spine_chassis takes 2 or 3 axles, not {axles}")
    last = x + length - 6
    at = [x + 3] + ([last - 4] if axles == 3 else []) + [last]
    if at[1] - at[0] < 6:
        raise ValueError(
            f"truck_spine_chassis needs more length for {axles} axles, not {length}"
        )
    # The holders' top, a whole 4 plates up: the wheels ride 0.75 LDU low on their pins, within a pin's play, so the
    # tires stand on z and everything on the holders is on the plate grid.
    top = z + 4
    for k, ax in enumerate(at):
        with m.step(f"{title}: axle {k + 1}"):
            m.add("4488", "Light Bluish Gray", at=(ax, y + 0.3, top - 2), rot=2)
            m.add("4488", "Light Bluish Gray", at=(ax, y + 3, top - 2), rot=0)
            for cy, turn in ((y + 0.375, OUT_LEFT), (y + 5.625, OUT_RIGHT)):
                c = (ax + 1, cy, z + TIRE)
                _raw(m, "11208", rim, c, turn)
                _raw(m, "11209", "Black", c, turn)
    # The spans between and beyond the arches, each covered by plates across.
    spans, a = [], x
    for ax in at:
        if ax - 1 > a:
            spans.append((a, ax - 1))
        a = ax + 3
    spans.append((a, x + length))
    widest = max(spans, key=lambda p: p[1] - p[0])
    joint = x + length if length in SPINE else _joint(x, length, widest)
    with m.step(f"{title}: spine"):
        m.add(Plate(2, joint - x), color, at=(x, y + 2, top), rot=1)
        if joint < x + length:
            m.add(Plate(2, x + length - joint), color, at=(joint, y + 2, top), rot=1)
        for ax in at:
            m.add("3023b", color, at=(ax, y + 1, top), rot=1)
            m.add("3023b", color, at=(ax, y + 4, top), rot=1)
    with m.step(f"{title}: arches"):
        for ax in at:
            m.add("98282", arch, at=(ax - 1, y - 0.443, top + 1), rot=3)
            m.add("98282", arch, at=(ax - 1, y + 4, top + 1), rot=1)
    with m.step(f"{title}: plates across"):
        for s, e in spans:
            for b in range(s, e - 1, 2):
                m.add("3795", color, at=(b, y, top + 1))
            if (e - s) % 2:
                m.add("3666", color, at=(e - 1, y, top + 1))
    # The keel: the longest brick that fits the widest span, over the spine's joint.
    s, e = widest
    n = max(b for b in BRICKS if b <= e - s)
    k = min(max(s, (joint if joint < x + length else (s + e) // 2) - n // 2), e - n)
    with m.step(f"{title}: keel"):
        m.add(BRICKS[n], color, at=(k, y + 2, top + 2), rot=1)
    return top + 2


def example(m):
    """A 6-wide truck chassis 20 long with a tandem rear axle."""
    m.use("truck_spine_chassis", x=0, y=0, z=0, length=20, axles=3)
