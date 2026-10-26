# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model


@model
def obsolete():
    """A plate with a moved part and an obsolete part on it (validate: two obsolete_part warnings)."""
    m = Model()
    m.add("3031", "Green", at=(0, 0, 0), id="base")
    m.add("3023", "Red", at=(0, 0, 1), id="moved")
    m.add("3660a", "Red", at=(2, 0, 1), id="obsolete")
    return m
