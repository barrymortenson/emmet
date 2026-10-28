# SPDX-License-Identifier: Apache-2.0
"""emmet params --write-default rewrites one literal through ast positions; every other byte must stay (plan 3B)."""

import unittest

from emmet_dsl.errors import EmmetError
from emmet_dsl.rewrite import write_default


def rewrite(source: str, param: str, value, kind: str) -> str:
    new, _ = write_default(source.encode("utf-8"), param, value, kind, "model.py")
    return new.decode("utf-8")


class WriteDefaultTest(unittest.TestCase):
    def check(self, before: str, param: str, value, kind: str, old: str, new: str):
        """Only the one literal changes: the result is `before` with that occurrence of `old` replaced."""
        after = rewrite(before, param, value, kind)
        at = after.index(new)
        self.assertEqual(after[:at] + old + after[at + len(new) :], before)

    def test_a_multi_line_signature_with_comments_and_annotated_ranges(self):
        before = '''from typing import Annotated, Literal
from emmet_dsl import Color, Model, Range, Step, model

WIDE = 12


@model
def station(
    length: Annotated[int, Range(16, 64), Step(8)] = 32,  # platform length, 32 by default
    # how tall, in bricks
    height: Annotated[float, Range(0.5, 4)]=1.5,
    canopy: Literal["flat", 'gable'] = 'flat',
    roof_color: Color = "Dark Bluish Gray",
    *,
    lamps: bool = True,
    offset: int = -3,
):
    """A station.  length: 32 is not this one"""
    return Model()
'''
        self.check(before, "length", 48, "int", "= 32,", "= 48,")
        self.check(before, "height", 2.5, "float", "=1.5,", "=2.5,")
        self.check(before, "height", 2, "float", "=1.5,", "=2.0,")
        self.check(before, "canopy", "gable", "choice", "'flat',", "'gable',")
        self.check(before, "roof_color", "Dark Red", "color", '"Dark Bluish Gray"', '"Dark Red"')
        self.check(before, "lamps", False, "bool", "= True,", "= False,")
        self.check(before, "offset", 4, "int", "= -3,", "= 4,")
        self.check(before, "offset", -7, "int", "= -3,", "= -7,")

    def test_quotes_and_non_ascii_text_before_the_literal(self):
        before = 'from emmet_dsl import model\n\n\n@model\ndef café(note: str = "it\'s ‘here’", w: int = 8): ...\n'
        after = rewrite(before, "w", 10, "int")
        self.assertEqual(after, before.replace("w: int = 8", "w: int = 10"))
        after = rewrite(before, "note", 'say "hi"', "choice")
        self.assertEqual(after, before.replace('"it\'s ‘here’"', '"say \\"hi\\""'))

    def test_a_byte_order_mark_stays(self):
        # Some Windows editors save one; the rewrite used to fail on it with a SyntaxError at line 1.
        before = "\ufefffrom emmet_dsl import model\n\n@model\ndef m(w: int = 8): ...\n"
        self.assertEqual(rewrite(before, "w", 9, "int"), before.replace("= 8", "= 9"))

    def test_the_last_model_function_is_the_one_rewritten(self):
        before = (
            "import emmet_dsl\n\n@emmet_dsl.model\ndef a(w: int = 1): ...\n\n"
            "def helper(w: int = 2): ...\n\n@emmet_dsl.model\ndef b(w: int = 3): ...\n"
        )
        self.assertEqual(rewrite(before, "w", 9, "int"), before.replace("w: int = 3", "w: int = 9"))

    def test_crlf_line_endings_stay(self):
        before = "from emmet_dsl import model\r\n\r\n@model\r\ndef m(\r\n    w: int = 8,\r\n):\r\n    ...\r\n"
        self.assertEqual(rewrite(before, "w", 12, "int"), before.replace("= 8", "= 12"))

    def test_a_default_that_isnt_a_literal_is_refused(self):
        before = "from emmet_dsl import model\nWIDE = 12\n\n@model\ndef m(w: int = WIDE, h: int = 2 * 3): ...\n"
        for name in ("w", "h"):
            with self.assertRaises(EmmetError) as e:
                rewrite(before, name, 10, "int")
            self.assertEqual(e.exception.code, "PARAM_INVALID")
            self.assertEqual(e.exception.line, 5)

    def test_an_unknown_parameter_is_refused(self):
        with self.assertRaises(EmmetError) as e:
            rewrite("from emmet_dsl import model\n\n@model\ndef m(w: int = 1): ...\n", "depth", 2, "int")
        self.assertEqual(e.exception.code, "PARAM_INVALID")


if __name__ == "__main__":
    unittest.main()
