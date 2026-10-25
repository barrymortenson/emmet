# SPDX-License-Identifier: Apache-2.0
"""EmmetError and how errors find the user's own file and line (plan section 08)."""

from __future__ import annotations

import os
import sys

_PACKAGE_DIR = os.path.dirname(os.path.abspath(__file__))


class EmmetError(Exception):
    """An error with a code from plan section 09, a hint, and the user's file and line."""

    def __init__(self, code: str, message: str, hint: str, file: str | None = None, line: int | None = None):
        super().__init__(message)
        self.code = code
        self.message = message
        self.hint = hint
        self.file = file
        self.line = line


def in_dsl(filename: str) -> bool:
    """True for frames inside emmet_dsl itself."""
    return os.path.abspath(filename).startswith(_PACKAGE_DIR + os.sep)


def caller() -> tuple[str, int]:
    """The (absolute file, line) of the first frame outside emmet_dsl: the model code that called in."""
    frame = sys._getframe(1)
    while frame is not None and in_dsl(frame.f_code.co_filename):
        frame = frame.f_back
    if frame is None:
        return ("", 0)
    return (frame.f_code.co_filename, frame.f_lineno)
