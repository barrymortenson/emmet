# SPDX-License-Identifier: Apache-2.0
"""The DSL vocabulary as Markdown, from emmet_dsl's own signatures and docstrings: what `emmet api` prints.

`python -m emmet_dsl.api` writes it to stdout. crates/emmet-cli/doctrine/api.md is this output, and a test fails when
the committed file is stale (tests/test_api_doc.py).
"""

from __future__ import annotations

import inspect

import emmet_dsl

HEADER = """# emmet_dsl API

Generated from emmet_dsl's docstrings (`python -m emmet_dsl.api`); don't edit.

A model.py imports from emmet_dsl, marks one function with `@model`, and returns a `Model`. The function's
parameters are the model's params: each needs a default, and its type annotation picks the control (`int` or
`float` with `Range` and `Step`, `bool`, `Literal[...]` for a choice, `Color`). The docstring's `name: help` lines
are each param's help text. Units: x runs right and y back, in studs; z runs up, in plates (3 plates make a brick).
"""


def _signature(name: str, obj) -> str:
    try:
        sig = str(inspect.signature(obj, eval_str=True))
    except (NameError, TypeError, ValueError):
        return name
    return f"{name}{sig}".replace("emmet_dsl.core.", "").replace("emmet_dsl.", "")


def _doc(obj) -> str:
    return inspect.getdoc(obj) or ""


def _members(cls) -> list[tuple[str, object]]:
    """Public methods and properties defined on the class itself, in source order."""
    out = []
    for name, value in cls.__dict__.items():
        if name.startswith("_"):
            continue
        if isinstance(value, property) or inspect.isfunction(value):
            out.append((name, value))
    return out


def render() -> str:
    lines = [HEADER]
    for name in emmet_dsl.__all__:
        if name.startswith("__"):
            continue
        obj = getattr(emmet_dsl, name)
        if inspect.isclass(obj):
            lines.append(f"## class {name}\n")
            if _doc(obj):
                lines.append(_doc(obj) + "\n")
            for member, value in _members(obj):
                if isinstance(value, property):
                    lines.append(f"### {name}.{member} (property)\n")
                    doc = _doc(value.fget)
                else:
                    sig = _signature(member, value).replace("(self, ", "(").replace("(self)", "()")
                    lines.append(f"### {name}.{sig}\n")
                    doc = _doc(value)
                if doc:
                    lines.append(doc + "\n")
        else:
            lines.append(f"## {_signature(name, obj)}\n")
            if _doc(obj):
                lines.append(_doc(obj) + "\n")
    return "\n".join(lines).rstrip() + "\n"


if __name__ == "__main__":
    print(render(), end="")
