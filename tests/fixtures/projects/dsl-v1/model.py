# SPDX-License-Identifier: AGPL-3.0-or-later
from typing import Literal

from emmet_dsl import (
    Brick,
    Color,
    Model,
    Plate,
    Rot,
    Slope,
    Tile,
    import_ldraw,
    model,
)


@model
def yard(
    roof: Literal["slope", "flat"] = "slope",
    wall_color: Color = "Tan",
    width: float = 6.0,
    gate: bool = True,
):
    """Every v1 call once: a walled yard with a hut, a stacked post, a tiled path, and an imported cart.
    roof: the hut's roof
    wall_color: the walls' color
    width: unused here; a float param
    gate: leave a gap in the front wall
    """
    m = Model()
    with m.step("Ground"):
        m.add("3811", "Green", at=(0, 0, 0), id="base")
    for row in range(2):
        with m.step(f"Walls, row {row + 1}"):
            m.wall(
                x=2,
                y=2,
                z=0.5 + 3 * row,
                length=10,
                depth=8,
                color=wall_color,
                openings=[(6, 2), (7, 2)] if gate else (),
            )
    with m.step("Path"):
        m.row(Tile, "Light Bluish Gray", at=(14, 2, 0.5), length=8, axis="y")
        m.fill(Plate, "Dark Bluish Gray", at=(16, 2, 0.5), size=(4, 4))
        m.add(Tile(2, 2), "Tan", at=(16, 6, 0.5))
    with m.step("Posts"):
        m.grid(Brick(1, 1), "Black", at=(22, 2, 0.5), nx=2, ny=2, rot=Rot(z=90))
        m.stack("3062b", "Black", at=(26, 2, 0.5), count=3)
    with m.step("Lamps"):
        m.repeat(
            lambda i: m.add("3062b", "Yellow", at=(22, 6, 0.5), id=f"lamp/{i}"), 3, dx=2
        )
    with m.mirror("x", about=16):
        with m.step("Corner stones"):
            m.add(Slope(45, 2, 1), "Dark Bluish Gray", at=(0, 20, 0.5), rot=2)
    with m.step("Hut roof"):
        if roof == "slope":
            m.use(
                "gable_roof",
                x=2,
                y=12,
                z=0.5,
                length=4,
                depth=4,
                color="Dark Red",
                title="Hut roof",
            )
        else:
            m.add("3031", "Dark Red", at=(2, 12, 0.5))
    with m.step("Flag"):
        flag = m.add_raw("3024", "Red", [270, -60, -250, 1, 0, 0, 0, 1, 0, 0, 0, 1])
    m.allow("floating", flag, "a flag on a pole the cart carries in the story")
    m.ground("base")
    cart = import_ldraw("wagon.ldr")
    with m.step("Cart"):
        m.place(cart, at=(24, 24, 0.5))
    return m
