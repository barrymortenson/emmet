# SPDX-License-Identifier: Apache-2.0
"""`emmet api` prints crates/emmet-cli/doctrine/api.md, generated from emmet_dsl's docstrings; it must match them.

Regenerate: uv run --project python/emmet_dsl python -m emmet_dsl.api > crates/emmet-cli/doctrine/api.md
"""

import unittest
from pathlib import Path

from emmet_dsl import api

COMMITTED = Path(__file__).resolve().parents[3] / "crates" / "emmet-cli" / "doctrine" / "api.md"


class ApiDocTest(unittest.TestCase):
    def test_the_committed_api_doc_is_fresh(self):
        self.assertEqual(
            COMMITTED.read_text(encoding="utf-8"),
            api.render(),
            "doctrine/api.md is stale; regenerate it (see this file's docstring)",
        )
