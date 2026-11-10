# SPDX-License-Identifier: Apache-2.0
from emmet_dsl import Model, model


@model
def build():
    """An empty model, ready to build: no baseplate, so it stands on the table, at z = 0."""
    m = Model()
    return m
