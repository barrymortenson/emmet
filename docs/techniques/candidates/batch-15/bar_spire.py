# SPDX-License-Identifier: Apache-2.0
"""Bar spire: a needle antenna or spire far thinner than a stud, a bar standing in a hollow stud with round plates
threaded on it as rings.

m.use("bar_spire", x, y, z, length=4, base="round_brick", rings=1, color="Light Bluish Gray", bar_color="Light Bluish
Gray", title="Spire") stands, on the stud at (x, y) at height z, a round 1 x 1 brick with a hollow stud (3062b), or for
base "round_plate" a round 1 x 1 plate with an open stud (85861), and a bar in it: a bar 3L (87994) for length 3 or a
bar 4L (30374) for length 4, its foot a plate above the base's underside. `rings` round plates with open studs (85861, 0
to 3) are threaded on the bar, each a brick above the one below, starting a brick over the base, so the spire tapers in
steps. Returns the z of the bar's tip.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066 and set 21061. Not
derived from any model file."""

PARTS = ["3062b", "85861", "87994", "30374"]

BARS = {
    3: ("87994", 7.5),
    4: ("30374", 10),
}  # bar by length in studs: part and length in plates
BASES = {"round_brick": ("3062b", 3), "round_plate": ("85861", 1)}


def build(
    m,
    x,
    y,
    z,
    length=4,
    base="round_brick",
    rings=1,
    color="Light Bluish Gray",
    bar_color="Light Bluish Gray",
    title="Spire",
):
    """The base, the bar, and its rings; returns the z of the tip."""
    if length not in BARS or base not in BASES or not 0 <= rings <= 3:
        raise ValueError(
            f"bar_spire takes a length of 3 or 4, a base of {' or '.join(BASES)}, and 0 to 3 rings, "
            f"not {length}, {base!r}, {rings}"
        )
    bar, long = BARS[length]
    part, high = BASES[base]
    foot = z + (
        1 if high > 1 else 0
    )  # into the round brick's hollow stud, or through the open-stud plate
    tip = foot + long
    if rings and z + high + 3 * rings + 1 > tip - 1:
        raise ValueError(f"bar_spire: a bar {length}L holds fewer than {rings} rings")
    with m.step(f"{title}: base and bar"):
        m.add(part, color, at=(x, y, z))
        m.add(bar, bar_color, at=(x + 0.3, y + 0.3, foot))
    if rings:
        with m.step(f"{title}: rings"):
            for k in range(rings):
                m.add("85861", color, at=(x, y, z + high + 3 * k + 2))
    return tip


def example(m):
    """A short tower with a spire of two rings, and a plain needle on a lower roof beside it."""
    from emmet_dsl import Brick, Plate, Tile

    with m.step("Ground"):
        m.add(Plate(4, 4), "Dark Bluish Gray", at=(0, 0, 0))
    with m.step("Tower"):
        m.stack(Brick(2, 2), "White", at=(0, 1, 1), count=4)
        m.stack(Brick(1, 2), "White", at=(2, 1, 1), count=2)
    with m.step("Roofs"):
        m.add(Tile(1, 2), "Light Bluish Gray", at=(0, 2, 13), rot=1)
        m.add(Tile(1, 1), "Light Bluish Gray", at=(1, 1, 13))
        m.add(Tile(1, 1), "Light Bluish Gray", at=(2, 2, 7))
    m.use(
        "bar_spire",
        x=0,
        y=1,
        z=13,
        length=4,
        rings=2,
        color="White",
        title="Main spire",
    )
    m.use(
        "bar_spire",
        x=2,
        y=1,
        z=7,
        length=3,
        base="round_plate",
        rings=0,
        title="Needle",
    )
