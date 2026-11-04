# SPDX-License-Identifier: AGPL-3.0-or-later
from emmet_dsl import Model, model

# (part, color, x, y, z, rot): solid, transparent, pearl, chrome, metal, rubber, and glowing parts, slopes, and round
# parts on a plate.
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
    # A second plate for the metals, rubber, and glow.
    ("3958", "Light Bluish Gray", 6, 0, 0, 0),
    ("6141", "Chrome Silver", 7, 1, 1, 0),
    ("3062b", "Chrome Gold", 9, 1, 1, 0),
    ("3005", "Metallic Silver", 11, 1, 1, 0),
    ("4032a", "Rubber Black", 7, 3, 1, 0),
    ("3062b", "Glow In Dark Opaque", 10, 4, 1, 0),
]


@model
def colors():
    """A small still life of finishes for the viewer's visual baseline."""
    m = Model()
    for i, (part, color, x, y, z, rot) in enumerate(PARTS):
        m.add(part, color, at=(x, y, z), rot=rot, id=f"p{i}")
    return m
