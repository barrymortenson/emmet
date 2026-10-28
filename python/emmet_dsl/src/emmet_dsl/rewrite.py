# SPDX-License-Identifier: Apache-2.0
"""`emmet params --write-default`: one parameter's default rewritten in model.py through the positions Python's ast
gives, so every other byte of the file stays as it was: formatting, comments, and the rest of the signature."""

from __future__ import annotations

import ast

from .errors import EmmetError


def _is_model(decorator: ast.expr) -> bool:
    target = decorator.func if isinstance(decorator, ast.Call) else decorator
    return (isinstance(target, ast.Name) and target.id == "model") or (
        isinstance(target, ast.Attribute) and target.attr == "model"
    )


def _model_function(tree: ast.Module) -> ast.FunctionDef | None:
    """The @model function emmet run uses: the last one the module defines."""
    found = [
        node
        for node in tree.body
        if isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef) and any(map(_is_model, node.decorator_list))
    ]
    return found[-1] if found else None


def _default(fn: ast.FunctionDef, name: str) -> ast.expr | None:
    args = fn.args
    positional = args.posonlyargs + args.args
    for arg, default in zip(positional[len(positional) - len(args.defaults) :], args.defaults, strict=True):
        if arg.arg == name:
            return default
    for arg, default in zip(args.kwonlyargs, args.kw_defaults, strict=True):
        if arg.arg == name:
            return default
    return None


def _literal(node: ast.expr) -> bool:
    if isinstance(node, ast.Constant):
        return isinstance(node.value, bool | int | float | str)
    return (
        isinstance(node, ast.UnaryOp)
        and isinstance(node.op, ast.USub | ast.UAdd)
        and isinstance(node.operand, ast.Constant)
        and isinstance(node.operand.value, int | float)
        and not isinstance(node.operand.value, bool)
    )


def _format(value, kind: str, old: str) -> str:
    """The new literal, written the way the old one was: a float stays a float, a string keeps its quote marks."""
    if kind == "bool" or isinstance(value, bool):
        return "True" if value else "False"
    if kind == "int":
        return str(int(value))
    if kind == "float":
        return repr(float(value))
    if isinstance(value, int | float):
        return repr(value)
    quote = "'" if old.lstrip("rRuU").startswith("'") else '"'
    body = str(value).replace("\\", "\\\\").replace(quote, "\\" + quote).replace("\n", "\\n")
    return quote + body + quote


def write_default(source: bytes, param: str, value, kind: str, file: str) -> tuple[bytes, int]:
    """`source` with `param`'s default replaced by `value`, and the line it's on."""
    # A byte-order mark (some Windows editors save one) stays; Python ignores it, and so does the parse.
    bom = b"\xef\xbb\xbf" if source.startswith(b"\xef\xbb\xbf") else b""
    text = source[len(bom) :].decode("utf-8")
    tree = ast.parse(text, filename=file)
    fn = _model_function(tree)
    if fn is None:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"{file} has no @model function.",
            "Mark the model's entry function with @model.",
            file,
            None,
        )
    node = _default(fn, param)
    if node is None:
        raise EmmetError(
            "PARAM_INVALID",
            f"{fn.name}() has no parameter '{param}' with a default.",
            "emmet params lists the model's parameters.",
            file,
            fn.lineno,
        )
    if not _literal(node):
        raise EmmetError(
            "PARAM_INVALID",
            f"{param}'s default isn't a plain value, so Emmet can't rewrite it.",
            f'Write the default as a literal (8, 2.5, True, "Tan") in {file}, then try again.',
            file,
            node.lineno,
        )
    # ast's columns count UTF-8 bytes, so the edit is made on the bytes.
    lines = source[len(bom) :].splitlines(keepends=True)
    start = len(bom) + sum(len(x) for x in lines[: node.lineno - 1]) + node.col_offset
    end = len(bom) + sum(len(x) for x in lines[: node.end_lineno - 1]) + node.end_col_offset
    old = source[start:end].decode("utf-8")
    new = _format(value, kind, old).encode("utf-8")
    return source[:start] + new + source[end:], node.lineno
