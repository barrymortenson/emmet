# SPDX-License-Identifier: Apache-2.0
"""Item launcher: a twin flick-fire launcher pod for a kart's nose or a turret, each barrel loaded with a round tile
as the item it fires.

m.use("item_launcher", x, y, z, facing=0, color="Light Bluish Gray", trigger="Dark Bluish Gray", items=("Green",
"Red"), title="Launcher") builds, on the studs at height z over (x, y) to (x + 2, y + 2), a 2 x 2 brick and on it two
plates 1 x 2 with a mini shooting blaster (69754) side by side, each with its trigger (69755) and a round tile 1 x 1
(98138) loaded in the barrel in one of `items`; a flick of the trigger fires the tile. The barrels reach half a stud
past the brick toward `facing` (quarter turns from +x: 0 fires to +x, 3 to the front, -y). It's a submodel. Returns
the placed copy's Handle.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72036. Not derived from any
model file."""

PARTS = ["3003", "69754", "69755", "98138"]

# The trigger and the loaded tile from the launcher's LDraw origin (the middle of its plate's top), as in the part's
# own assembly: offset and turn.
TRIGGER = ((0, -16.5, -25.3), (1, 0, 0, 0, 0.70711, -0.70711, 0, 0.70711, 0.70711))
ITEM = ((4, -6.9, -39.6), (0, -1, 0, 1, 0, 0, 0, 0, 1))


def _launcher(m, color, trigger, items):
    """The pod firing to +x, its brick over (0, 0) to (2, 2)."""
    with m.step("Brick"):
        m.add("3003", color, at=(0, 0, 0))
    with m.step("Launchers"):
        for k in range(2):
            # A launcher's plate over (0, k) to (2, k + 1) on the brick; its LDraw origin is the middle of the plate's
            # top.
            ox, oy, oz = 20 * (k + 0.5), -8 * 4, -20 * 1
            m.add_raw("69754", color, (ox, oy, oz, 1, 0, 0, 0, 1, 0, 0, 0, 1))
            loaded = (
                ("69755", trigger, TRIGGER),
                ("98138", items[k % len(items)], ITEM),
            )
            for part, tint, ((dx, dy, dz), turn) in loaded:
                m.add_raw(part, tint, (ox + dx, oy + dy, oz + dz, *turn))


def build(
    m,
    x,
    y,
    z,
    facing=0,
    color="Light Bluish Gray",
    trigger="Dark Bluish Gray",
    items=("Green", "Red"),
    title="Launcher",
):
    """The pod; returns the placed copy's Handle."""
    sub = m.submodel(title, _launcher, color=color, trigger=trigger, items=tuple(items))
    (x0, y0, z0), (x1, y1, _) = sub.box
    # The box's corner after turning the pod about the middle of its brick, (x + 1, y + 1).
    corners = [(cx - 1, cy - 1) for cx in (x0, x1) for cy in (y0, y1)]
    for _ in range(facing % 4):
        corners = [(-cy, cx) for cx, cy in corners]
    lo = (min(c[0] for c in corners) + 1, min(c[1] for c in corners) + 1)
    with m.step(title):
        return m.place(sub, at=(x + lo[0], y + lo[1], z + z0), rot=facing % 4)


def example(m):
    """Two pods on a deck, one firing right and one firing front."""
    from emmet_dsl import Plate

    with m.step("Deck"):
        m.fill(Plate, "Blue", at=(0, 0, 0), size=(6, 4))
    m.use("item_launcher", x=4, y=1, z=1)
    m.use(
        "item_launcher",
        x=0,
        y=1,
        z=1,
        facing=3,
        items=("Yellow",),
        title="Second launcher",
    )
