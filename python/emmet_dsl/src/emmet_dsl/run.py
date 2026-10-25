# SPDX-License-Identifier: Apache-2.0
"""`python -I -m emmet_dsl.run`: runs a project's model for emmet run (plan sections 07 and 08).

emmet sends one JSON request on stdin and reads one JSON reply line from stdout. The model's own prints go to stderr,
which emmet keeps as build/run.log. The DSL never reads the LDraw library: emmet passes the body boxes of the parts the
model uses in build/part-geometry.json. When the model asks for a part that isn't there, this pass reports the
missing ids with their call sites and writes nothing; emmet looks them up and runs again.
"""

from __future__ import annotations

import faulthandler
import importlib.util
import json
import os
import sys
import time
import traceback

from . import core as model_mod
from . import params as params_mod
from .core import Model, suggest
from .emit import write_all
from .errors import EmmetError, in_dsl


def color_key(name: str) -> str:
    """The same key emmet's color table uses: lowercase letters and digits, "grey" spelled "gray"."""
    return "".join(c for c in name.lower() if c.isascii() and c.isalnum()).replace("grey", "gray")


class Context:
    """What Model.add needs from emmet: part geometry, the color table, and where to note missing parts."""

    def __init__(self, root: str, geometry: dict, colors: list):
        self.root = root
        self.geometry = geometry
        self.colors = {c["code"]: c for c in colors}
        self.color_keys = {}
        for c in colors:
            for n in [c["name"], *c.get("aliases", [])]:
                self.color_keys.setdefault(color_key(n), c["code"])
        self.missing: dict[str, tuple[str, int]] = {}

    def rel(self, file: str) -> str:
        path = os.path.abspath(file)
        if path.startswith(self.root + os.sep):
            return os.path.relpath(path, self.root).replace(os.sep, "/")
        return file

    def part_id(self, part, file: str, line: int) -> str:
        if not isinstance(part, str) or not part.strip():
            raise EmmetError(
                "UNKNOWN_PART",
                f"{part!r} isn't a part id.",
                'Pass an LDraw part id as a string, like "3001" (Brick 2 x 4). Find ids with emmet search.',
                file,
                line,
            )
        pid = part.strip().replace("\\", "/").lower()
        return pid[:-4] if pid.endswith(".dat") else pid

    def body(self, part: str, file: str, line: int) -> list:
        found = self.geometry.get(part)
        if found is None:
            self.missing.setdefault(part, (file, line))
            return [0, 0, 0, 0, 0, 0]
        return found["body"]

    def color(self, value, file: str, line: int) -> int:
        if isinstance(value, int) and not isinstance(value, bool) and value in self.colors:
            return value
        if isinstance(value, str):
            text = value.strip()
            if text.isdigit() and int(text) in self.colors:
                return int(text)
            code = self.color_keys.get(color_key(text))
            if code is not None:
                return code
        names = [c["name"] for c in self.colors.values()]
        close = suggest(str(value), names) or suggest(str(value).title(), names)
        maybe = f"Did you mean {_either(close)}? " if close else ""
        raise EmmetError(
            "UNKNOWN_COLOR",
            f"There's no color {value!r}.",
            f"{maybe}Use an LDraw, Rebrickable, or BrickLink color name, or an LDraw code; emmet colors lists them.",
            file,
            line,
        )


def _either(names: list[str]) -> str:
    """'a', 'a' or 'b', 'a', 'b', or 'c'."""
    quoted = [f"'{n}'" for n in names]
    if len(quoted) <= 2:
        return " or ".join(quoted)
    return ", ".join(quoted[:-1]) + ", or " + quoted[-1]


def _excerpt(root: str, file: str, line: int | None) -> str | None:
    if not line:
        return None
    try:
        with open(os.path.join(root, file), encoding="utf-8") as f:
            for n, text in enumerate(f, 1):
                if n == line:
                    return text.rstrip("\n").rstrip()
    except OSError:
        return None
    return None


def _error(root: str, code: str, message: str, hint: str, file: str | None, line: int | None) -> dict:
    details = {}
    if file:
        details["file"] = file
    if line:
        details["line"] = line
    excerpt = _excerpt(root, file, line) if file else None
    if excerpt is not None:
        details["excerpt"] = excerpt
    return {"ok": False, "error": {"code": code, "message": message, "hint": hint, "details": details}}


def _from_exception(root: str, ctx: Context, exc: BaseException) -> dict:
    if isinstance(exc, EmmetError):
        file = ctx.rel(exc.file) if exc.file else exc.file
        return _error(root, exc.code, exc.message, exc.hint, file, exc.line)
    if isinstance(exc, SystemExit):
        return _error(root, "MODEL_EXEC_FAILED", "The model called exit().", "Return the Model instead.", None, None)
    if isinstance(exc, SyntaxError) and exc.filename and not in_dsl(exc.filename):
        file = ctx.rel(exc.filename)
        msg = f"SyntaxError: {exc.msg}"
        return _error(root, "MODEL_EXEC_FAILED", msg, f"Fix {file} line {exc.lineno} and run again.", file, exc.lineno)
    # The innermost frame in the user's own files is where the fix goes.
    frames = [f for f in traceback.extract_tb(exc.__traceback__) if not in_dsl(f.filename)]
    frames = [f for f in frames if os.path.abspath(f.filename).startswith(root + os.sep)] or frames
    shown = "".join(traceback.TracebackException.from_exception(exc).format_exception_only()).strip()
    message, _, suggestion = shown.partition(". Did you mean: ")
    hint_lead = f"Did you mean {suggestion.rstrip('?')}? " if suggestion else ""
    if frames:
        f = frames[-1]
        file = ctx.rel(f.filename)
        return _error(
            root, "MODEL_EXEC_FAILED", message, f"{hint_lead}Fix {file} line {f.lineno} and run again.", file, f.lineno
        )
    return _error(root, "MODEL_EXEC_FAILED", message, f"{hint_lead}Fix the model and run again.", None, None)


def _load(path: str):
    spec = importlib.util.spec_from_file_location("model", path)
    module = importlib.util.module_from_spec(spec)
    sys.modules["model"] = module
    spec.loader.exec_module(module)
    return module


def _read_json(path: str, default):
    if not path or not os.path.exists(path):
        return default
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def main() -> int:
    reply_to = sys.stdout
    sys.stdout = sys.stderr  # the model's prints go to the log, not the reply
    request = json.load(sys.stdin)
    root = os.path.abspath(os.getcwd())
    timeout = request.get("timeout")
    if timeout:
        # Dumps the stack (with the model's current line) to stderr and exits; emmet reads it for MODEL_TIMEOUT.
        faulthandler.dump_traceback_later(float(timeout), exit=True)
    sys.path.insert(0, root)
    colors = _read_json(request.get("colors"), {"colors": []})["colors"]
    geometry = _read_json(request.get("geometry"), {"parts": {}})["parts"]
    ctx = Context(root, geometry, colors)
    model_mod.active = ctx
    try:
        reply = _run(request, ctx)
    except (Exception, SystemExit) as exc:  # every failure in the model becomes an envelope
        reply = _from_exception(root, ctx, exc)
    reply_to.write(json.dumps(reply) + "\n")
    reply_to.flush()
    return 0


def _run(request: dict, ctx: Context) -> dict:
    entry = request.get("entry", "model.py")
    module = _load(os.path.join(ctx.root, entry))
    fn = next((f for f in reversed(model_mod.registry) if f.__module__ == module.__name__), None)
    if fn is None:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"{entry} has no @model function.",
            "Mark the model's entry function with @model (from emmet_dsl import model).",
            entry,
            None,
        )
    file = ctx.rel(fn.__code__.co_filename)
    def_line = fn.__code__.co_firstlineno
    schema = params_mod.schema(fn, file, def_line)
    values = params_mod.check(schema, request.get("params", {}), file, request.get("params_source", "params"))
    for p in schema:
        if p.type == "color":
            ctx.color(values[p.name], file, p.line)
    doc = (fn.__doc__ or "").strip().splitlines()
    title = doc[0] if doc else ""
    if request.get("mode") == "schema":
        data = {"model": fn.__name__, "doc": title, "file": file, "params": [p.to_json() for p in schema]}
        return {"ok": True, "data": {**data, "values": values}}
    started = time.perf_counter()
    result = fn(**values)
    if not isinstance(result, Model):
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"{fn.__name__}() returned {type(result).__name__}, not a Model.",
            "End the @model function with `return m`, where m = Model().",
            file,
            def_line,
        )
    if ctx.missing:
        return {"ok": False, "need": [{"part": p, "file": f, "line": n} for p, (f, n) in ctx.missing.items()]}
    ms = int((time.perf_counter() - started) * 1000)
    summary = write_all(request["out"], fn.__name__, title, result, request.get("manifest", {}), ms)
    return {"ok": True, "data": summary}


if __name__ == "__main__":
    sys.exit(main())
