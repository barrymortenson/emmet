# SPDX-License-Identifier: Apache-2.0
"""Shipping container: a 20-foot freight container for a flat wagon, a truck bed, or a port stack, corrugated panel
sides on a plate base that lifts off.

m.use("shipping_container", x, y, z, color="Dark Turquoise", post="Light Bluish Gray", title="Container") builds a
container 8 studs long (along x) and 6 wide on the studs at (x, y, z): a 6 x 8 plate as its floor; at each corner a
post of five 1 x 1 bricks; along each long side a corrugated panel 1 x 6 x 5 (23405) between the posts; across each
end five 1 x 4 bricks; and a 6 x 8 plate roof, tiled smooth, with a round 1 x 1 plate on each corner post as the
corner casting another container stacks on. Its floor only sits on the studs under it, so it lifts off. Returns the
z of the roof's top (the tiles).
"""

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 60509. Not derived from any
model file."""

PARTS = ["3036", "3005", "23405", "3010", "87079", "2431", "6141"]


def build(
    m, x, y, z, color="Dark Turquoise", post="Light Bluish Gray", title="Container"
):
    """The floor, the posts and walls, and the roof; returns the z of the roof's top."""
    with m.step(f"{title}: floor"):
        m.add("3036", color, at=(x, y, z), rot=1)
    with m.step(f"{title}: side panels"):
        m.add("23405", color, at=(x + 1, y, z + 1), rot=3)
        m.add("23405", color, at=(x + 1, y + 5, z + 1), rot=1)
    for r in range(5):
        with m.step(f"{title}: posts and ends, row {r + 1}"):
            for cx in (x, x + 7):
                for cy in (y, y + 5):
                    m.add("3005", post, at=(cx, cy, z + 1 + 3 * r))
                m.add("3010", color, at=(cx, y + 1, z + 1 + 3 * r), rot=0)
    with m.step(f"{title}: roof"):
        m.add("3036", color, at=(x, y, z + 16), rot=1)
    with m.step(f"{title}: roof tiles"):
        m.fill(Tile, color, at=(x + 1, y, z + 17), size=(6, 6))
    with m.step(f"{title}: roof ends"):
        for cx in (x, x + 7):
            m.add("2431", color, at=(cx, y + 1, z + 17), rot=0)
            for cy in (y, y + 5):
                m.add("6141", post, at=(cx, cy, z + 17))
    return z + 18


def example(m):
    """A container on a 6 x 10 plate, as on a wagon's deck."""
    with m.step("Deck"):
        m.add("3033", "Reddish Brown", at=(-1, 0, 0), rot=1)
    m.use("shipping_container", x=0, y=0, z=1)
