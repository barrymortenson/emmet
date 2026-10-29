# SPDX-License-Identifier: Apache-2.0
"""Shop window: a shopfront window built as a unit, tall panes with small transom lights over them.

m.use("shop_window", x, y, z, panes=3, frame="Dark Green", title="Shop window") builds, along x from (x, y, z) and
facing front (-y), a sill plate 2 x panes long, then `panes` windows 1 x 2 x 3 with glass, a plate across their tops,
and a window 1 x 2 x 2 with glass over each as its transom. Returns the z of its top.
"""

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350. Not derived
from any model file."""

PARTS = ["60593", "60602", "60592", "60601", "3666", "3460", "3710", "3023b", "4477"]

PLATES = {2: "3023b", 4: "3710", 6: "3666", 8: "3460", 10: "4477"}  # Plate 1 x n


def _glass(m, f, glass, size):
    (x0, y0, z0), (x1, y1, _) = f.min, f.max
    m.add(
        glass,
        "Trans-Clear",
        at=((x0 + x1 - size) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
        rot=1,
    )


def build(m, x, y, z, panes=3, frame="Dark Green", title="Shop window"):
    """The sill, the panes, and the transoms; returns the z of its top."""
    n = 2 * panes
    if n not in PLATES:
        raise ValueError(f"shop_window takes 1 to 5 panes, not {panes}")
    with m.step(f"{title}: sill and panes"):
        m.add(PLATES[n], frame, at=(x, y, z), rot=1)
        for k in range(panes):
            _glass(
                m, m.add("60593", frame, at=(x + 2 * k, y, z + 1), rot=1), "60602", 1.65
            )
    with m.step(f"{title}: transoms"):
        m.add(PLATES[n], frame, at=(x, y, z + 10), rot=1)
        for k in range(panes):
            _glass(
                m,
                m.add("60592", frame, at=(x + 2 * k, y, z + 11), rot=1),
                "60601",
                1.65,
            )
    return z + 17


def example(m):
    """A three-pane shop window on its own."""
    m.use("shop_window", x=0, y=0, z=0)
