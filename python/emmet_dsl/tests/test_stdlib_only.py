# SPDX-License-Identifier: Apache-2.0
"""emmet_dsl imports nothing outside the standard library (plan section 06), so a model never needs pip install."""

import ast
import sys
import unittest
from pathlib import Path

PACKAGE = Path(__file__).resolve().parents[1] / "src" / "emmet_dsl"


class StdlibOnlyTest(unittest.TestCase):
    def test_every_import_is_stdlib_or_the_package(self):
        files = sorted(PACKAGE.glob("**/*.py"))
        self.assertGreater(len(files), 5)
        outside = []
        for path in files:
            for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
                if isinstance(node, ast.Import):
                    names = [a.name for a in node.names]
                elif isinstance(node, ast.ImportFrom) and node.level == 0:
                    names = [node.module or ""]
                else:
                    continue
                for name in names:
                    top = name.split(".")[0]
                    if top not in sys.stdlib_module_names and top not in ("emmet_dsl", "__future__"):
                        outside.append(f"{path.name}: {name}")
        self.assertEqual(outside, [])


if __name__ == "__main__":
    unittest.main()
