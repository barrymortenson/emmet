# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model

# (part, color, x, y, z, rot): solid, transparent, and pearl parts, slopes, and round parts on a plate.
PARTS = [
    ("3958", "Light Bluish Gray", 0, 0, 0, 0),
    ("3001", "Red", 0, 0, 1, 0),
    ("3001", "Blue", 2, 0, 1, 0),
    ("3003", "Yellow", 4, 0, 1, 0),
    ("3039", "Dark Red", 0, 0, 4, 0),
    ("3039", "Dark Red", 2, 0, 4, 0),
    ("3065", "Trans-Clear", 4, 2, 1, 1),
    ("3065", "Trans-Red", 0, 4, 1, 0),
    ("3065", "Trans-Dark Blue", 2, 4, 1, 0),
    ("3062b", "Trans-Green", 4, 4, 1, 0),
    ("4589", "Pearl Gold", 5, 4, 1, 0),
    ("6141", "Trans-Yellow", 4, 5, 1, 0),
    ("3068b", "White", 0, 6, 1, 0),
    ("3024", "Black", 5, 5, 1, 0),
]


@model
def colors():
    """A small still life of finishes for the viewer's visual baseline."""
    m = Model()
    for i, (part, color, x, y, z, rot) in enumerate(PARTS):
        m.add(part, color, at=(x, y, z), rot=rot, id=f"p{i}")
    return m
