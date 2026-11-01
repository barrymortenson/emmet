# SPDX-License-Identifier: Apache-2.0
"""fit (3E): a technique's z carrying attributes reads as the number, but two with one number and different attributes
never key the same submodel (submodels and builders are keyed by repr; the review of PR #49)."""

import unittest

from emmet_dsl.core import fit


class FitTest(unittest.TestCase):
    def test_reads_as_the_number(self):
        top = fit(30, door_top=18)
        self.assertEqual(top + 1, 31)
        self.assertEqual(str(top), "30")
        self.assertEqual(f"{top}", "30")
        self.assertEqual(top.door_top, 18)
        self.assertEqual(str(fit(4.5, underside=3)), "4.5")

    def test_attributes_tell_two_values_apart(self):
        self.assertNotEqual(repr(fit(30, door_top=18)), repr(fit(30, door_top=21)))
        self.assertEqual(repr(fit(30)), "30")


if __name__ == "__main__":
    unittest.main()
