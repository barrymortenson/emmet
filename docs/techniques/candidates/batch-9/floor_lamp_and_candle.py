# SPDX-License-Identifier: Apache-2.0
"""Floor lamp and candle: a standing lamp with a shade on a slim pole, or a candle in a brass dish, built apart and
placed.

m.use("floor_lamp_and_candle", x, y, z, kind="floor_lamp", pole="Tan", shade="White", wax="White", title=None) stands
the piece on the studs at height z with its min corner at (x, y). kind="floor_lamp" is 2 x 2: a round 2 x 2 plate with
one middle stud as the foot, a round 1 x 1 plate with an open stud on it, a bar 3L in `pole` standing in that stud, and
a cone in `shade` on the bar's top as the shade, 11.5 plates tall. kind="candle" is 1 x 1: a round plate in pearl gold
as the dish, a round brick in `wax`, and a flame, 1 brick and 1 plate tall before the flame; put one on a mantel, a
table, a shelf, or a piano. Both are round, so they have no facing. It's a submodel, built once in its own step,
"<title>" ("Floor lamp" or "Candle" by default), placed in a step of the same name; it returns the placed copy's Handle.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43217, set 21335. Not derived
from any model file."""

PARTS = ["18674", "85861", "87994", "4589", "6141", "3062b", "37775"]


def _lamp(m, pole, shade, title):
    """The lamp in its own frame: 2 x 2, its pole in the middle at (1, 1)."""
    with m.step(title):
        m.add("18674", "Dark Bluish Gray", at=(0, 0, 0), id="foot")
        # The foot's one stud is in its middle, so the lamp stands half a stud off the grid.
        m.add("85861", "Dark Bluish Gray", at=(0.5, 0.5, 1), id="socket")
        m.add("87994", pole, at=(0.8, 0.8, 2), id="pole")
        # The bar reaches a plate up into the cone.
        m.add("4589", shade, at=(0.5, 0.5, 8.5), id="shade")


def _candle(m, wax, title):
    """The candle in its own frame: 1 x 1."""
    with m.step(title):
        m.add("6141", "Pearl Gold", at=(0, 0, 0), id="dish")
        m.add("3062b", wax, at=(0, 0, 1), id="candle")
        m.add("37775", "Trans-Orange", at=(0.14, 0.3, 4), id="flame")


def build(
    m, x, y, z, kind="floor_lamp", pole="Tan", shade="White", wax="White", title=None
):
    """The lamp or candle; returns the placed copy's Handle."""
    if kind == "floor_lamp":
        sub = m.submodel(
            "floor lamp", _lamp, pole=pole, shade=shade, title=title or "Floor lamp"
        )
    elif kind == "candle":
        sub = m.submodel("candle", _candle, wax=wax, title=title or "Candle")
    else:
        raise ValueError(f'kind must be "floor_lamp" or "candle", not {kind!r}')
    with m.step(title or kind.replace("_", " ").capitalize()):
        return m.place(sub, at=(x, y, z))


def example(m):
    """A floor lamp beside a stool with a candle on it."""
    with m.step("Floor"):
        m.add("3031", "Medium Nougat", at=(0, 0, 0))
    m.use("floor_lamp_and_candle", x=0, y=2, z=1)
    with m.step("Stool"):
        m.add("3003", "Reddish Brown", at=(2, 0, 1))
    m.use("floor_lamp_and_candle", x=2, y=0, z=4, kind="candle")
