# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def figure_clearance_clean():
    """A shop front built the way the building rules ask, with no findings: the base figure_clearance's mutations
    start from. A door under a canopy that clears it, two benches centered between the canopy's columns, and a second
    door under a hood."""
    m = Model()
    with m.step("Base"):
        m.add("3811", "Green", at=(0, 0, 0), id="base")
    m.use(
        "door_frame",
        x=2,
        y=12,
        z=0.5,
        length=20,
        rows=8,
        offset=2,
        facing="front",
        title="Shop",
    )
    office = m.use(
        "door_frame",
        x=24,
        y=12,
        z=0.5,
        length=6,
        rows=6,
        offset=1,
        facing="front",
        title="Office",
    )
    canopy = m.use("canopy", x=2, y=4, z=0.5, length=20, depth=8, spacing=9)
    with m.step("Benches"):
        xs = sorted({cx for cx, _ in canopy.columns})
        for k, (a, b) in enumerate(zip(xs, xs[1:])):
            m.use("bench", at=((a + b + 1) // 2 - 2, 5, 0.5), id=f"bench/{k + 1}")
    m.use("door_hood", x=25, y=12, z=office.door_top, width=4)
    return m
