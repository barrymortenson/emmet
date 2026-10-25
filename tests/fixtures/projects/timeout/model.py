# SPDX-License-Identifier: AGPL-3.0-or-later
import time

from emmet_dsl import Model, model


@model
def forever():
    """A model that doesn't finish in time: line 10 waits an hour; emmet.toml allows 1 s."""
    time.sleep(3600)
    return Model()
