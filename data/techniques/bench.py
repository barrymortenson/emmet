# SPDX-License-Identifier: Apache-2.0
"""Bench: a park or platform bench, 4 studs long and 2 deep, built apart as a sub-assembly and placed.

m.use("bench", at=(x, y, z), rot=0, color="Reddish Brown", legs="Dark Bluish Gray", id=None) places one bench with
the minimum corner of its box at `at`, on studs: two legs, a seat with a tile to sit on, and a lattice back. At rot=0
it runs along x and faces −y (its back is at the larger y); rot turns it like any part. Every bench with the same
colors is one sub-assembly, built once in its own steps and placed "× n", so place them inside your own step. It
returns the placed copy's Handle.
"""

PARTS = ["3004", "3020", "2431", "3633"]


def _bench(m, color, legs):
    with m.step("Legs and seat"):
        m.add("3004", legs, at=(0, 0, 0), id="leg/left")
        m.add("3004", legs, at=(3, 0, 0), id="leg/right")
        m.add("3020", color, at=(0, 0, 3), rot=1, id="seat")
    with m.step("Seat and back"):
        m.add("2431", color, at=(0, 0, 4), rot=1, id="seat/tile")
        m.add("3633", color, at=(0, 1, 4), rot=1, id="back")


def build(m, at, rot=0, color="Reddish Brown", legs="Dark Bluish Gray", id=None):
    bench = m.submodel("Bench", _bench, color=color, legs=legs)
    return m.place(bench, at=at, rot=rot, id=id)


def example(m):
    """Two benches facing each other on a plate."""
    with m.step("Plate"):
        m.add("3031", "Light Bluish Gray", at=(0, 0, 0))
    with m.step("Benches"):
        m.use("bench", at=(0, 2, 1))
        m.use("bench", at=(0, 0, 1), rot=2)
