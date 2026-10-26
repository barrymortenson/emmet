# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def alias():
    """A plate with an alias part on it (validate: one alias_part, info)."""
    m = Model()
    m.add("3031", "Green", at=(0, 0, 0), id="base")
    m.add("15570", "Yellow", at=(1, 1, 1), id="eye")
    return m
