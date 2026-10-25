# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def forever():
    """A model that never finishes: the loop on line 10. emmet.toml sets a 1 s limit."""
    m = Model()
    n = 0
    while n >= 0:
        n += 1  # fmt: skip  (one line, so the timeout always lands on line 10)
    return m
