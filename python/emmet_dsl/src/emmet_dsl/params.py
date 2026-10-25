# SPDX-License-Identifier: Apache-2.0
"""The param schema from the @model function's signature, and checking values against it (plan section 08)."""

from __future__ import annotations

import ast
import difflib
import inspect
import textwrap
import typing
from dataclasses import dataclass
from typing import Annotated, Any, Literal

from .errors import EmmetError


@dataclass(frozen=True)
class Range:
    """Annotated[int, Range(6, 16)]: the lowest and highest value a slider allows."""

    min: float
    max: float


@dataclass(frozen=True)
class Step:
    """Annotated[float, Step(0.5)]: the slider's increment."""

    step: float


class Color(str):
    """A color parameter: a name in any spelling ("Light Bluish Gray", "LBG") or an LDraw code."""


@dataclass
class Param:
    name: str
    type: str  # int, float, bool, choice, color
    default: Any
    line: int
    min: float | None = None
    max: float | None = None
    step: float | None = None
    choices: list | None = None
    help: str = ""

    def to_json(self) -> dict:
        out = {"name": self.name, "type": self.type, "default": self.default}
        for key in ("min", "max", "step", "choices"):
            value = getattr(self, key)
            if value is not None:
                out[key] = value
        out["help"] = self.help
        out["line"] = self.line
        return out


def _help_lines(doc: str | None) -> dict[str, str]:
    """`name: help text` lines from the docstring."""
    found = {}
    for line in (doc or "").splitlines():
        name, sep, text = line.strip().partition(":")
        if sep and name.isidentifier():
            found[name] = text.strip()
    return found


def _param_lines(fn) -> dict[str, int]:
    """The line each parameter is declared on, in the model's own file."""
    try:
        lines, start = inspect.getsourcelines(fn)
        tree = ast.parse(textwrap.dedent("".join(lines)))
    except (OSError, SyntaxError, IndentationError):
        return {}
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef):
            args = node.args.posonlyargs + node.args.args + node.args.kwonlyargs
            return {a.arg: start + a.lineno - 1 for a in args}
    return {}


def schema(fn, file: str, def_line: int) -> list[Param]:
    try:
        hints = typing.get_type_hints(fn, include_extras=True)
    except NameError as e:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"A parameter annotation of {fn.__name__}() names something that isn't defined: {e}.",
            "Import it (for example `from typing import Annotated`) or fix the name.",
            file,
            def_line,
        ) from e
    help_text = _help_lines(fn.__doc__)
    lines = _param_lines(fn)
    params = []
    for p in inspect.signature(fn).parameters.values():
        line = lines.get(p.name, def_line)
        if p.default is inspect.Parameter.empty:
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"Parameter '{p.name}' has no default value.",
                f"Give it one, for example `{p.name}: int = 8`. Every parameter becomes a slider with a default.",
                file,
                line,
            )
        hint = hints.get(p.name, type(p.default))
        extras: tuple = ()
        if typing.get_origin(hint) is Annotated:
            hint, *rest = typing.get_args(hint)
            extras = tuple(rest)
        param = Param(p.name, _kind(hint, p.name, file, line), p.default, line, help=help_text.get(p.name, ""))
        if param.type == "choice":
            param.choices = list(typing.get_args(hint))
        for extra in extras:
            if isinstance(extra, Range):
                param.min, param.max = extra.min, extra.max
            elif isinstance(extra, Step):
                param.step = extra.step
        params.append(param)
    return params


def _kind(hint, name: str, file: str, line: int) -> str:
    if hint is bool:
        return "bool"
    if hint is int:
        return "int"
    if hint is float:
        return "float"
    if hint is Color:
        return "color"
    if typing.get_origin(hint) is Literal:
        return "choice"
    raise EmmetError(
        "MODEL_EXEC_FAILED",
        f"Parameter '{name}' has type {getattr(hint, '__name__', hint)!r}, which can't be a slider.",
        "Use int, float, bool, Color, or Literal[...] (a choice), with Annotated[int, Range(lo, hi)] for a range.",
        file,
        line,
    )


def check(params: list[Param], values: dict, file: str, source: str) -> dict:
    """The effective values: defaults with `values` on top, each checked against its type and range.

    `source` says where the values came from (params.json, --params), for the message.
    """
    by_name = {p.name: p for p in params}
    out = {p.name: p.default for p in params}
    for name, value in values.items():
        p = by_name.get(name)
        if p is None:
            close = difflib.get_close_matches(name, list(by_name), n=1)
            maybe = f"Did you mean '{close[0]}'? " if close else ""
            known = ", ".join(by_name) or "none"
            where, line = _key_line("params.json", name) if source.startswith("params.json") else (None, None)
            raise EmmetError(
                "PARAM_INVALID",
                f"There's no parameter '{name}' (from {source}).",
                f"{maybe}The model's parameters are: {known}.",
                where or file,
                line or min((q.line for q in params), default=None),
            )
        out[name] = _coerce(p, value, file, source)
    for p in params:
        _check_default(p, file)
    return out


def _key_line(path: str, key: str) -> tuple[str | None, int | None]:
    """The file and line where `"key"` appears in a JSON file in the project, if it does."""
    try:
        with open(path, encoding="utf-8") as f:
            for n, text in enumerate(f, 1):
                if f'"{key}"' in text:
                    return (path, n)
    except OSError:
        pass
    return (None, None)


def _bad(p: Param, value, file: str, source: str, why: str) -> EmmetError:
    rng = f" from {_num(p.min)} to {_num(p.max)}" if p.min is not None else ""
    if p.step:
        rng += f" in steps of {_num(p.step)}"
    if p.type == "choice":
        rng = ": " + ", ".join(repr(c) for c in p.choices or [])
    kind = {"int": "a whole number", "float": "a number", "bool": "true or false", "choice": "one of the choices"}
    kind |= {"color": "a color name or code"}
    return EmmetError(
        "PARAM_INVALID",
        f"{p.name} = {value!r} (from {source}) {why}.",
        f"Use {kind.get(p.type, 'a value')}{rng}; the default is {p.default!r}.",
        file,
        p.line,
    )


def _on_step(p: Param, num) -> bool:
    """On the slider's steps from its minimum, allowing for float error (0.3 is on steps of 0.1)."""
    if not p.step:
        return True
    k = (num - (p.min or 0)) / p.step
    return abs(k - round(k)) < 1e-9


def _num(v) -> str:
    return str(int(v)) if float(v).is_integer() else str(v)


def _coerce(p: Param, value, file: str, source: str):
    if p.type == "bool":
        if isinstance(value, bool):
            return value
        if isinstance(value, str) and value.lower() in ("true", "false", "1", "0", "yes", "no", "on", "off"):
            return value.lower() in ("true", "1", "yes", "on")
        raise _bad(p, value, file, source, "isn't true or false")
    if p.type in ("int", "float"):
        num = value
        if isinstance(value, str):
            try:
                num = float(value)
            except ValueError:
                raise _bad(p, value, file, source, "isn't a number") from None
        if isinstance(num, bool) or not isinstance(num, int | float):
            raise _bad(p, value, file, source, "isn't a number")
        if p.type == "int":
            if not float(num).is_integer():
                raise _bad(p, value, file, source, "isn't a whole number")
            num = int(num)
        if p.min is not None and not (p.min <= num <= p.max):
            raise _bad(p, value, file, source, f"is outside {_num(p.min)}–{_num(p.max)}")
        if not _on_step(p, num):
            raise _bad(p, value, file, source, f"isn't on the slider's steps of {_num(p.step)}")
        return num
    if p.type == "choice":
        for c in p.choices or []:
            if c == value or str(c) == str(value):
                return c
        raise _bad(p, value, file, source, "isn't one of the choices")
    if p.type == "color":
        if isinstance(value, int | str) and not isinstance(value, bool):
            return value
        raise _bad(p, value, file, source, "isn't a color name or code")
    return value


def _check_default(p: Param, file: str) -> None:
    if p.min is not None and isinstance(p.default, int | float) and not (p.min <= p.default <= p.max):
        raise EmmetError(
            "PARAM_INVALID",
            f"{p.name}'s default {p.default!r} is outside its range {_num(p.min)}–{_num(p.max)}.",
            "Change the default or the Range so the default fits.",
            file,
            p.line,
        )
    if isinstance(p.default, int | float) and not isinstance(p.default, bool) and not _on_step(p, p.default):
        raise EmmetError(
            "PARAM_INVALID",
            f"{p.name}'s default {p.default!r} isn't on its steps of {_num(p.step)}.",
            "Change the default or the Step so the default is one of the slider's values.",
            file,
            p.line,
        )
