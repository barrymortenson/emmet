# SPDX-License-Identifier: AGPL-3.0-or-later
"""Walls for the submodule fixture."""


def wall(m, length):
    for x in range(0, length, 2):
        m.add("3001", "Red", at=(x, 0, 0))
    # The bad color is on the next line, line 9.
    m.add("3001", "Purpel", at=(0, 0, 3))
