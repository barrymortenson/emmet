# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def huge():
    """A model that places 20,001 plates; the last one is too many (line 11)."""
    m = Model()
    for i in range(20_001):
        x, y = i % 32, (i // 32) % 32
        m.add("3024", "Red", at=(x, y, i // 1024))
    return m
