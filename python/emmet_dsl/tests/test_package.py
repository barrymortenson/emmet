# SPDX-License-Identifier: Apache-2.0
"""Imports emmet_dsl the way a model runs: an isolated interpreter (python -I) in a fresh process."""

import subprocess
import sys
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class PackageTest(unittest.TestCase):
    def test_version_matches_pyproject(self):
        expected = tomllib.loads((ROOT / "pyproject.toml").read_text())["project"]["version"]
        out = subprocess.run(
            [sys.executable, "-I", "-c", "import emmet_dsl; print(emmet_dsl.__version__)"],
            capture_output=True,
            text=True,
            check=True,
        )
        self.assertEqual(out.stdout.strip(), expected)


if __name__ == "__main__":
    unittest.main()
