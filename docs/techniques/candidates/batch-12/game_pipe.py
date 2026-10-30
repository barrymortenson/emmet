# SPDX-License-Identifier: Apache-2.0
"""Game pipe: a game's green pipe, a column of round bricks under a wider round lip with a dark opening on top.

m.use("game_pipe", x, y, z, height=3, size="small", color="Green", inside="Black", title="Pipe") stands a pipe on the
studs at height z, its column over (x, y): for size="small", round bricks 2 x 2 (3941), `height` bricks tall, then a
lip a stud wider all round, a round brick 4 x 4 (87081) over (x - 1, y - 1) to (x + 3, y + 3), with a round plate 4 x
4 with a round hole (11833) on it as the rim and a round tile 2 x 2 (14769) in `inside` in the hole as the dark
opening; for size="large", round bricks 4 x 4 (87081) as the column, then three round plates 6 x 6 (11213) over (x -
1, y - 1) to (x + 5, y + 5) as the lip and a round plate 4 x 4 (60474) in `inside` on it as the opening, the lip's
ring of studs left bare round it. Returns the z of the top.
"""

PROVENANCE = """Emmet's own (session 3D), from the video-game style rules drawn from the instruction books of set
71374 and set 72036 (a pipe appears there only in pixel art). Not derived from any model file."""

PARTS = ["3941", "87081", "11833", "14769", "11213", "60474"]

SIZES = ("small", "large")


def build(
    m, x, y, z, height=3, size="small", color="Green", inside="Black", title="Pipe"
):
    """The column, the lip, and the opening; returns the z of the top."""
    if size not in SIZES:
        raise ValueError(
            f"game_pipe: size must be one of {', '.join(SIZES)}, not {size!r}"
        )
    if height < 1:
        raise ValueError(f"game_pipe needs a height of 1 or more bricks, not {height}")
    column = "3941" if size == "small" else "87081"
    for k in range(0, height, 4):
        n = min(4, height - k)
        with m.step(f"{title}: column" + (f" {k // 4 + 1}" if height > 4 else "")):
            m.stack(column, color, at=(x, y, z + 3 * k), count=n)
    zl = z + 3 * height
    with m.step(f"{title}: lip"):
        if size == "small":
            m.add("87081", color, at=(x - 1, y - 1, zl))
            m.add("11833", color, at=(x - 1, y - 1, zl + 3))
            m.add("14769", inside, at=(x, y, zl + 3))
            return zl + 4
        m.stack("11213", color, at=(x - 1, y - 1, zl), count=3)
        m.add("60474", inside, at=(x, y, zl + 3))
    return zl + 4


def example(m):
    """A small pipe and a large one on a patch of sand."""
    from emmet_dsl import Plate

    with m.step("Grass"):
        m.fill(Plate, "Tan", at=(0, 0, 0), size=(12, 8))
    m.use("game_pipe", x=2, y=3, z=1, height=2)
    m.use("game_pipe", x=7, y=2, z=1, height=3, size="large", title="Large pipe")
