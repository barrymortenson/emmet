# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def unavailable():
    """Two 2 x 4 bricks in Violet, a color that brick was never made in (validate, with the parts data pack: one
    color_unavailable warning naming both)."""
    m = Model()
    m.add("3031", "Green", at=(0, 0, 0), id="base")
    m.add("3001", "Violet", at=(0, 0, 1), id="left")
    m.add("3001", "Violet", at=(2, 0, 1), id="right")
    m.add("3001", "Red", at=(1, 0, 4), id="top")
    return m
