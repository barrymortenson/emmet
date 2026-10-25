# SPDX-License-Identifier: Apache-2.0
"""The DSL a model.py imports to emit an .mpd (plan section 08). v0: @model, Model.add, Model.step, and parameters."""

__version__ = "0.1.0"

from .core import Handle, Model, model  # noqa: E402
from .errors import EmmetError  # noqa: E402
from .params import Color, Range, Step  # noqa: E402

__all__ = ["Color", "EmmetError", "Handle", "Model", "Range", "Step", "model", "__version__"]
