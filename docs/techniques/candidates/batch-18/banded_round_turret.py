# SPDX-License-Identifier: Apache-2.0
"""Banded round turret: a slim round corner turret of 2 x 2 round bricks with a dark recessed ring and a slight lip
every few bricks, the string courses of a midi-scale tower.

m.use("banded_round_turret", x, y, z, bricks=6, band_every=3, color="Tan", band="Black", roof="Sand Green",
title="Turret") stands a turret 2 x 2 studs on the studs at height z, its corner at (x, y): `bricks` 2 x 2 round bricks
(3941), and after every `band_every` of them a band: four 1 x 1 round plates in `band` on the brick's studs (the dark
ring, set in) under a 2 x 2 round plate in `color` (the lip). A 2 x 2 round plate caps it, and with roof a cone
2 x 2 x 2 in that color on top (None leaves a round tile). It's a submodel, built once however many corners use it.
Returns the placed copy's Handle; its .max[2] is the top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21067. Not derived from any
model file."""

PARTS = ["3941", "6141", "4032a", "3942c", "14769"]


def _turret(m, bricks, band_every, color, band, roof, title):
    """One turret in its own frame, its corner at the origin."""
    zz, k = 0, 0
    while k < bricks:
        run = min(band_every or bricks, bricks - k)
        with m.step(f"{title}, bricks {k + 1}-{k + run}"):
            m.stack("3941", color, at=(0, 0, zz), count=run)
        zz, k = zz + 3 * run, k + run
        if k < bricks and band_every:
            with m.step(f"{title}, band at brick {k}"):
                m.grid("6141", band, at=(0, 0, zz), nx=2, ny=2)
                m.add("4032a", color, at=(0, 0, zz + 1))
            zz += 2
    with m.step(f"{title}, cap"):
        m.add("4032a", color, at=(0, 0, zz))
        if roof:
            m.add("3942c", roof, at=(0, 0, zz + 1))
        else:
            m.add("14769", color, at=(0, 0, zz + 1))


def build(
    m,
    x,
    y,
    z,
    bricks=6,
    band_every=3,
    color="Tan",
    band="Black",
    roof="Sand Green",
    title="Turret",
):
    """The turret, built apart and placed; returns the placed copy's Handle."""
    if not 1 <= bricks <= 20 or band_every < 0:
        raise ValueError(
            f"banded_round_turret takes 1 to 20 bricks and a band_every of 0 or more, not {bricks}, {band_every}"
        )
    turret = m.submodel(
        "turret",
        _turret,
        bricks=bricks,
        band_every=band_every,
        color=color,
        band=band,
        roof=roof,
        title=title,
    )
    with m.step(title):
        return m.place(turret, at=(x, y, z))


def example(m):
    """Two turrets flanking a wall."""
    from emmet_dsl import Brick, Plate

    with m.step("Ground"):
        m.add("3795", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("banded_round_turret", x=0, y=0, z=1, title="Left turret")
    m.use("banded_round_turret", x=4, y=0, z=1, title="Right turret")
    for r in range(4):
        with m.step(f"Wall, row {r + 1}"):
            m.add(Brick(1, 2), "Tan", at=(2, 0, 1 + 3 * r), rot=1)
            m.add(Brick(1, 2), "Tan", at=(2, 1, 1 + 3 * r), rot=1)
    with m.step("Wall, top"):
        m.add(Plate(2, 2), "Tan", at=(2, 0, 13))
