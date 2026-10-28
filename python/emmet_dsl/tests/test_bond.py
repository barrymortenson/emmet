# SPDX-License-Identifier: Apache-2.0
"""Family.bond, which lays each course of Model.wall over the one below: no joint on a joint, fewest pieces."""

import unittest

from emmet_dsl import core
from emmet_dsl.parts import Brick

# Bricks 1 x n the library has (no 1 x 5, 1 x 7, or 1 x 9).
HAVE = {1, 2, 3, 4, 6, 8, 10, 12, 16}


class Library:
    def rel(self, file):
        return file

    def helper(self, kind, dims, soft, rel, line):
        w, n, _ = dims
        return f"1x{n}" if w == 1 and n in HAVE else None


def joints(start, pieces):
    out, at = [], start
    for n, _ in pieces[:-1]:
        at += n
        out.append(at)
    return out


class BondTest(unittest.TestCase):
    def setUp(self):
        core.active = Library()

    def tearDown(self):
        core.active = None

    def test_no_joint_lands_on_a_joint_below(self):
        below = {4, 8, 12}  # a 16-stud course laid 1 x 4s
        pieces = Brick.bond(0, 16, below, None, "model.py", 1)
        self.assertEqual(sum(n for n, _ in pieces), 16)
        self.assertFalse(set(joints(0, pieces)) & below, pieces)

    def test_fewest_pieces_and_no_one_by_ones_when_it_can(self):
        # Over a 1 x 8 and a 1 x 2 (joint at 8), with nothing longer than a 1 x 8, a 10-stud run takes two pieces
        # that meet away from 8, not a 1 x 1.
        pieces = Brick.bond(0, 10, {8}, [1, 2, 3, 4, 6, 8], "model.py", 1)
        self.assertEqual(len(pieces), 2, pieces)
        self.assertNotIn(8, joints(0, pieces))
        self.assertNotIn(1, [n for n, _ in pieces])

    def test_the_course_over_a_window_bridges_its_edges(self):
        # The course below stops at a window from 3 to 5: its run ends are joints, so the course over it bridges.
        pieces = Brick.bond(0, 8, {3, 5}, None, "model.py", 1)
        self.assertFalse({3, 5} & set(joints(0, pieces)), pieces)

    def test_positions_are_absolute(self):
        pieces = Brick.bond(10, 6, {12, 14}, None, "model.py", 1)
        self.assertFalse({12, 14} & set(joints(10, pieces)), pieces)

    def test_a_run_that_cant_avoid_them_falls_back_to_longest_first(self):
        # 1 x 1s everywhere below: every joint is taken, so it lays the run as it would with nothing below.
        self.assertEqual(Brick.bond(0, 3, {1, 2}, None, "model.py", 1), [(3, "1x3")])
        self.assertEqual(Brick.bond(0, 1, {0, 1}, None, "model.py", 1), [(1, "1x1")])


if __name__ == "__main__":
    unittest.main()
