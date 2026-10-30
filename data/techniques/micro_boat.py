# SPDX-License-Identifier: Apache-2.0
"""Micro boat: a ferry or tug a stud wide, a plate hull with a cheese slope at each end and a deck house in the middle,
for a micro harbor.

m.use("micro_boat", x, y, z, length=4, axis="x", hull="White", house="White", cabin="Black", title="Boat") stands, on
the studs at height z from (x, y), a boat `length` studs long (4 to 6) along `axis` ("x" or "y"): a 1 x n plate as the
hull, a cheese slope (54200) on each end stud facing out as the bow and the stern, and on the studs between, a deck
house: a plate topped with tiles, with a 1 x 1 tile in the cabin color as the wheelhouse windows. Set it on the bare
studs left in a water field (water_tile_field's `studs`), so its hull sits flush with the water. Returns the z of its
top.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = ["3710", "3666", "3023b", "3623", "54200", "3069b", "3070b", "63864", "2431"]


def build(
    m,
    x,
    y,
    z,
    length=4,
    axis="x",
    hull="White",
    house="White",
    cabin="Black",
    title="Boat",
):
    """The hull, the bow and stern, and the deck house; returns the z of the top."""
    if not 4 <= length <= 6 or axis not in ("x", "y"):
        raise ValueError(
            f"micro_boat takes a length of 4 to 6 and an axis of 'x' or 'y', not {length}, {axis!r}"
        )

    def at(u, h=0):
        return (x + u, y, z + h) if axis == "x" else (x, y + u, z + h)

    turn = 1 if axis == "x" else 0  # a 1 x n part's long side along the boat
    back, front = (
        (2, 0) if axis == "x" else (3, 1)
    )  # the slopes' turns: low edge out at each end
    with m.step(f"{title}: hull"):
        m.add(Plate(1, length), hull, at=at(0), rot=turn)
        m.add("54200", hull, at=at(0, 1), rot=back)
        m.add("54200", hull, at=at(length - 1, 1), rot=front)
    mid = length - 2
    with m.step(f"{title}: deck house"):
        m.add(Plate(1, mid), house, at=at(1, 1), rot=turn)
        if mid > 1:
            m.add(Tile(1, mid - 1), house, at=at(1, 2), rot=turn)
        m.add(Tile(1, 1), cabin, at=at(mid, 2))
    return z + 3


def example(m):
    """Two boats, a ferry and a tug, on a patch of harbor water."""
    with m.step("Harbor floor"):
        m.add(Plate(6, 8), "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    # Water tiles everywhere but the boats' studs: (y, x from, length) runs.
    runs = [
        (0, 0, 8),
        (1, 0, 1),
        (1, 7, 1),
        (2, 0, 8),
        (3, 0, 3),
        (3, 7, 1),
        (4, 0, 8),
        (5, 0, 8),
    ]
    for k in range(0, len(runs), 4):
        with m.step(f"Water {k // 4 + 1}"):
            for wy, wx, n in runs[k : k + 4]:
                m.row(
                    Tile,
                    "Trans Light Blue",
                    at=(wx, wy, 1),
                    length=n,
                    sizes=(8, 6, 4, 2, 1),
                )
    m.use("micro_boat", x=1, y=1, z=1, length=6, title="Ferry")
    m.use("micro_boat", x=3, y=3, z=1, length=4, hull="Red", title="Tug")
