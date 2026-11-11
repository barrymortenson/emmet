# SPDX-License-Identifier: AGPL-3.0-or-later
"""A Technic brace on a 5-12-13 triangle (4G): a 15-hole beam on the table, a 15-hole brace from its hole 1 rising
5 studs over 12 to the brace's hole 14, and a 7-hole beam level on the brace's top end, each pinned."""

from emmet_dsl import Hole, Model, model


@model
def brace(fault: int = 0):
    """A 5-12-13 brace with a beam on its top end.

    fault: 1 puts the top pin half a stud to one side of the brace's hole; 2 puts it in the top beam, pointing away
    from the brace, so the brace's hole 14 is empty
    """
    m = Model()
    with m.step("Base"):
        base = m.beam("32278", "Black", at=(0, 0.5, 1.25), run=14, id="base")
    with m.step("Brace"):
        m.insert("2780", "Black", base.hole(1), out="+y", id="pin/low")
        brace = m.beam("32278", "Red", at=(0, 1.5, 1.25), run=12, rise=5, id="brace")
    with m.step("Top"):
        h = brace.hole(14)
        if fault == 1:
            aside = (h.at[0] + 0.5, h.at[1], h.at[2])
            h = Hole(h.name, h.kind, aside, h.axis, h.length, h.part, h.id)
        if fault == 2:
            top = m.beam("32524", "Black", at=(12, 2.5, 13.75), run=6, id="top")
            m.insert("2780", "Black", top.hole(1), out="+y", id="pin/high")
        else:
            m.insert("2780", "Black", h, out="+y", id="pin/high")
            m.beam("32524", "Black", at=(12, 2.5, 13.75), run=6, id="top")
    return m
