# SPDX-License-Identifier: AGPL-3.0-or-later
import time
from typing import Annotated

from emmet_dsl import Model, Range, model

COLORS = ["Red", "Blue", "Yellow", "White", "Black", "Green"]


@model
def grid(
    count: Annotated[int, Range(1, 4000)] = 500,
    delay: Annotated[float, Range(0, 30)] = 0.0,
):
    """Stacked layers of 2 x 4 bricks, for the viewer's visual baseline, bench, and E2E specs.
    count: how many bricks
    delay: seconds to wait before building (the E2E specs kill a run mid-way)
    """
    time.sleep(delay)
    m = Model()
    for layer in range((count + 99) // 100):
        with m.step(f"Layer {layer + 1}"):
            for i in range(layer * 100, min(count, layer * 100 + 100)):
                row, col = divmod(i % 100, 10)
                color = COLORS[(row + col + layer) % len(COLORS)]
                m.add("3001", color, at=(2 * col, 4 * row, 3 * layer), id=f"b{i}")
    return m
