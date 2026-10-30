# SPDX-License-Identifier: Apache-2.0
"""Cross display stand: a freestanding plus-shaped shop table with a poster on the end of each arm and goods on top,
built apart and placed.

m.use("cross_display_stand", x, y, z, color="Reddish Brown", posters="Bright Light Orange", goods=None, seed=0,
title="Display stand") stands a stand 6 x 6 studs on the studs at height z, its min corner at (x, y): four arms 2 x 2
round a middle 2 x 2, the plus a shop sets in an aisle. Each arm is a brick 1 x 2 x 1 2/3 with studs on one side at its
outer end, studs out, carrying a 2 x 2 tile as a poster in `posters`, and a brick and two plates behind it. Two layers
of plates make the top, a 2 x 6 and two 2 x 2s each, the second crossing the first so the four arms are one piece; 2 x 2
tiles finish the arms, and four round bricks stand in the middle as goods, their colors from `goods` (a list) or picked
from `seed`. The top is 8 plates up. The posters stand 0.4 studs proud of the 6 x 6. It's a submodel, built once in its
own steps, "<title>, arms", "<title>, arm plates", "<title>, posters", "<title>, top", and "<title>, goods", placed in a
step titled "<title>"; it returns the placed copy's Handle.
"""

import random

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = ["22885", "3004", "3023b", "3068b", "3795", "3022", "3062b"]

GOODS = ["Red", "Yellow", "Medium Azure", "Lime", "Bright Pink", "Orange", "White"]

# Each arm: (min corner, which way its poster faces, the poster's turn). A 2 x 2 tile on its back faces -y; turning it
# about z faces it the other ways.
ARMS = [
    ((2, 0), "-y", Rot(x=90)),
    ((4, 2), "+x", Rot(x=90, z=90)),
    ((2, 4), "+y", Rot(x=90, z=180)),
    ((0, 2), "-x", Rot(x=90, z=270)),
]


def _arm(m, ax, ay, face, color, layer):
    """One arm's bricks (layer 0: the side-stud brick at its outer end and a brick behind it) or its plates (layer 1:
    two plates on the brick behind)."""
    if face == "-y":
        outer, inner, rot = (ax, ay), (ax, ay + 1), (3, 1)
    elif face == "+y":
        outer, inner, rot = (ax, ay + 1), (ax, ay), (1, 1)
    elif face == "+x":
        outer, inner, rot = (ax + 1, ay), (ax, ay), (0, 0)
    else:
        outer, inner, rot = (ax, ay), (ax + 1, ay), (2, 0)
    if layer == 0:
        m.add("22885", color, at=(*outer, 0), rot=rot[0], id=f"arm{face}/end")
        m.add("3004", color, at=(*inner, 0), rot=rot[1], id=f"arm{face}/brick")
        return
    m.add("3023b", color, at=(*inner, 3), rot=rot[1], id=f"arm{face}/plate/1")
    m.add("3023b", color, at=(*inner, 4), rot=rot[1], id=f"arm{face}/plate/2")


def _poster_at(ax, ay, face):
    return {
        "-y": (ax, ay - 0.4),
        "+y": (ax, ay + 2),
        "+x": (ax + 2, ay),
        "-x": (ax - 0.4, ay),
    }[face]


def _stand(m, color, posters, goods, title):
    """The stand in its own frame: 6 x 6, the middle 2 x 2 at (2, 2)."""
    for layer, name in ((0, "arms"), (1, "arm plates")):
        with m.step(f"{title}, {name}"):
            for (ax, ay), face, _ in ARMS:
                _arm(m, ax, ay, face, color, layer)
    with m.step(f"{title}, posters"):
        for (ax, ay), face, turn in ARMS:
            m.add(
                "3068b",
                posters,
                at=(*_poster_at(ax, ay, face), 0),
                rot=turn,
                id=f"arm{face}/poster",
            )
    with m.step(f"{title}, top"):
        m.add("3795", color, at=(0, 2, 5), rot=1, id="top/1/across")
        m.add("3022", color, at=(2, 0, 5), id="top/1/front")
        m.add("3022", color, at=(2, 4, 5), id="top/1/back")
        m.add("3795", color, at=(2, 0, 6), id="top/2/along")
        m.add("3022", color, at=(0, 2, 6), id="top/2/left")
        m.add("3022", color, at=(4, 2, 6), id="top/2/right")
    with m.step(f"{title}, goods"):
        for (ax, ay), face, _ in ARMS:
            m.add("3068b", color, at=(ax, ay, 7), id=f"arm{face}/tile")
        for i, (gx, gy) in enumerate(((2, 2), (3, 2), (2, 3), (3, 3))):
            m.add("3062b", goods[i % len(goods)], at=(gx, gy, 7), id=f"goods/{i + 1}")


def build(
    m,
    x,
    y,
    z,
    color="Reddish Brown",
    posters="Bright Light Orange",
    goods=None,
    seed=0,
    title="Display stand",
):
    """The stand; returns the placed copy's Handle."""
    if goods is None:
        goods = random.Random(seed).sample(GOODS, 4)
    sub = m.submodel(
        "display stand",
        _stand,
        color=color,
        posters=posters,
        goods=tuple(goods),
        title=title,
    )
    with m.step(title):
        return m.place(sub, at=(x - 0.4, y - 0.4, z))


def example(m):
    """A stand in the middle of a shop floor."""
    with m.step("Floor"):
        m.add("3036", "Sand Green", at=(0, 0, 0), rot=1)
    m.use("cross_display_stand", x=1, y=0, z=1)
