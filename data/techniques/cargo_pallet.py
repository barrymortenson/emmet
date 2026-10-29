# SPDX-License-Identifier: Apache-2.0
"""Cargo pallet: a pallet with slots underneath for a forklift's forks, with crates stacked on it.

m.use("cargo_pallet", x, y, z, color="Reddish Brown", crate="Tan", title="Pallet") lays two plates 1 x 4 at (x, y, z)
and (x, y + 3, z) as the pallet's legs (leaving a 2-stud slot between them for the forks), a plate 4 x 4 across them,
and a brick 2 x 2 and two 1 x 2 bricks on it as crates. Returns the z of the pallet's top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 60440. Not derived from any model file."""

PARTS = ["3710", "3031", "3003", "3004"]


def build(m, x, y, z, color="Reddish Brown", crate="Tan", title="Pallet"):
    """The pallet and its crates; returns the z of the pallet's top."""
    with m.step(f"{title}: pallet"):
        for b in (0, 3):
            m.add("3710", color, at=(x, y + b, z), rot=1)
        m.add("3031", color, at=(x, y, z + 1))
    with m.step(f"{title}: crates"):
        m.add("3003", crate, at=(x, y, z + 2))
        m.add("3004", crate, at=(x + 2, y, z + 2))
        m.add("3004", crate, at=(x + 2, y + 2, z + 2))
    return z + 2


def example(m):
    """A pallet with its load."""
    m.use("cargo_pallet", x=0, y=0, z=0)
