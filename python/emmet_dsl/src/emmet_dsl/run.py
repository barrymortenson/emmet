# SPDX-License-Identifier: Apache-2.0
"""`python -I -B -X utf8 -m emmet_dsl.run`: runs a project's model for emmet run (plan sections 07 and 08).

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
from . import rewrite
from .core import Model, suggest
from .emit import write_all, write_atomic_bytes
from .errors import EmmetError, in_dsl


def color_key(name: str) -> str:
    """The same key emmet's color table uses: lowercase letters and digits, "grey" spelled "gray"."""
    return "".join(c for c in name.lower() if c.isascii() and c.isalnum()).replace("grey", "gray")


class Context:
    """What Model.add needs from emmet: part geometry, the color table, and where to note missing parts."""

    def __init__(self, root: str, geometry: dict, colors: list, helpers: dict | None = None, step_target: int = 8):
        self.root = root
        self.geometry = geometry
        self.colors = {c["code"]: c for c in colors}
        self.color_keys = {}
        for c in colors:
            for n in [c["name"], *c.get("aliases", [])]:
                self.color_keys.setdefault(color_key(n), c["code"])
        # The colors the parts data pack says no part was made in; a color it doesn't know (no "produced") stays open.
        self.unmade = {c["code"] for c in colors if c.get("produced") is False}
        self.helpers = helpers or {}  # "brick 2x4" → {"id": …, "near": […]}, as emmet resolved them
        self.missing: dict[str, tuple[str, int]] = {}
        self.missing_helpers: dict[str, dict] = {}
        self.count = 0  # parts placed so far, every submodel copy counted
        self.submodels: dict = {}  # (name, function, params) → Submodel, in the order they were made
        self.step_target = step_target
        # The @model function's (file, line), once it's known: where an error with no frame of its own points.
        self.model_at: tuple[str | None, int | None] = (None, None)

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

    def helper(self, kind: str, dims: tuple, soft: bool, file: str, line: int) -> str | None:
        """The part id for a helper (Brick(2, 4)), from emmet's answers. Before emmet has answered, the lookup is
        noted for the next pass and a placeholder stands in."""
        a, b, c = dims
        key = f"{kind} {a}x{b}" + (f"x{c}" if kind == "brick" and c != 1 else "") + (f" {c}" if kind == "slope" else "")
        if key not in self.helpers:
            self.missing_helpers.setdefault(key, {"kind": kind, "dims": list(dims), "file": file, "line": line})
            return f"?{key}"
        found = self.helpers[key]
        if found.get("id"):
            return found["id"]
        if soft:
            return None
        name = {"brick": "Brick", "plate": "Plate", "tile": "Tile", "slope": "Slope"}[kind]
        size = f"{c} {a} x {b}" if kind == "slope" else f"{a} x {b}" + (f" x {c}" if kind == "brick" and c != 1 else "")
        near = found.get("near") or []
        maybe = f"The library has {_either(near)}. " if near else ""
        raise EmmetError(
            "UNKNOWN_PART",
            f"There's no {name} {size} in the library.",
            f"{maybe}Use one of those sizes, or find a part with emmet search.",
            file,
            line,
        )

    def body(self, part: str, file: str, line: int) -> list:
        if part.startswith("?"):  # a helper emmet hasn't looked up yet
            return [0, 0, 0, 0, 0, 0]
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
            f"{maybe}Use an LDraw color name in any spelling, an abbreviation like LBG, or an LDraw code; emmet colors "
            "lists them.",
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
    """The text of `line` in `file`, read only when the file is inside the project: a model can raise an error that
    names any path."""
    path = os.path.realpath(os.path.join(root, file))
    if not line or not path.startswith(os.path.realpath(root) + os.sep):
        return None
    try:
        # errors="replace": a file saved in another encoding still gets its own error, not a crash here.
        with open(path, encoding="utf-8", errors="replace") as f:
            for n, text in enumerate(f, 1):
                if n == line:
                    return text.rstrip("\n").rstrip()
    except (OSError, ValueError):
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
    if isinstance(exc, SyntaxError) and exc.filename and not in_dsl(exc.filename):
        file = ctx.rel(exc.filename)
        msg = f"SyntaxError: {exc.msg}"
        return _error(root, "MODEL_EXEC_FAILED", msg, f"Fix {file} line {exc.lineno} and run again.", file, exc.lineno)
    # The innermost frame in the user's own files is where the fix goes. Frames in the standard library or in
    # emmet_dsl never are: with no frame in the project, point at the @model function, if there is one.
    frames = [
        f
        for f in traceback.extract_tb(exc.__traceback__)
        if not in_dsl(f.filename) and os.path.abspath(f.filename).startswith(root + os.sep)
    ]
    if isinstance(exc, SystemExit):
        message, hint_lead = "The model called exit().", "Return the Model instead of exiting. "
    else:
        shown = "".join(traceback.TracebackException.from_exception(exc).format_exception_only()).strip()
        message, _, suggestion = shown.partition(". Did you mean: ")
        hint_lead = f"Did you mean {suggestion.rstrip('?')}? " if suggestion else ""
    if frames:
        file, line = ctx.rel(frames[-1].filename), frames[-1].lineno
    else:
        file, line = ctx.model_at
    if file and line:
        return _error(
            root, "MODEL_EXEC_FAILED", message, f"{hint_lead}Fix {file} line {line} and run again.", file, line
        )
    return _error(root, "MODEL_EXEC_FAILED", message, f"{hint_lead}Fix the model and run again.", file, None)


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
    known = _read_json(request.get("geometry"), {"parts": {}})
    ctx = Context(root, known["parts"], colors, known.get("helpers"), request.get("step_target") or 8)
    model_mod.active = ctx
    try:
        reply = _run(request, ctx)
    except (Exception, SystemExit) as exc:  # every failure in the model becomes an envelope
        reply = _failure(root, ctx, exc)
    reply_to.write(json.dumps(reply) + "\n")
    reply_to.flush()
    return 0


def _failure(root: str, ctx: Context, exc: BaseException) -> dict:
    """The reply for a pass that raised. Parts the model asked for that emmet hasn't sent yet come first: until the
    model has their real sizes, an error that depends on them (a size divided into a length, say) isn't the model's."""
    if ctx.missing or ctx.missing_helpers:
        return _need(ctx)
    try:
        return _from_exception(root, ctx, exc)
    except Exception as inner:  # noqa: BLE001 - a reply must go out whatever happens here
        msg = f"emmet_dsl couldn't report {type(exc).__name__}: {exc} ({type(inner).__name__}: {inner})"
        return _error(root, "INTERNAL", msg, "This is a bug in Emmet; build/run.log has the details.", None, None)


def _need(ctx: Context) -> dict:
    """The parts and helpers this pass lacked, for emmet to look up before the next."""
    need = [{"part": p, "file": f, "line": n} for p, (f, n) in ctx.missing.items() if not p.startswith("?")]
    need += [{"helper": k, **v} for k, v in ctx.missing_helpers.items()]
    return {"ok": False, "need": need}


def _write_default(request: dict, ctx: Context) -> dict:
    """mode write_default: rewrites one default in model.py without running the model."""
    entry = request.get("entry", "model.py")
    path = os.path.join(ctx.root, entry)
    with open(path, "rb") as f:
        source = f.read()
    new, line = rewrite.write_default(source, request["param"], request["value"], request.get("kind", ""), entry)
    if new != source:
        write_atomic_bytes(path, new)
    return {"ok": True, "data": {"param": request["param"], "file": entry, "line": line}}


def _run(request: dict, ctx: Context) -> dict:
    if request.get("mode") == "write_default":
        return _write_default(request, ctx)
    entry = request.get("entry", "model.py")
    path = os.path.join(ctx.root, entry)
    if not os.path.isfile(path):
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"The project has no {entry}.",
            f"Create {entry} with an @model function, or run emmet new to start from a template.",
            None,
            None,
        )
    module = _load(path)
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
    ctx.model_at = (file, def_line)
    ctx.main_file = f"{fn.__name__}.ldr".lower()
    schema = params_mod.schema(fn, file, def_line)
    knobs = getattr(fn, "__emmet_knobs__", ())
    unknown = [k for k in knobs if k not in {p.name for p in schema}]
    if unknown:
        raise EmmetError(
            "PARAM_INVALID",
            f"@model(knobs=...) names {', '.join(unknown)}, which {fn.__name__}() has no parameter for.",
            f"Name parameters of {fn.__name__}(): {', '.join(p.name for p in schema) or 'it has none'}.",
            file,
            def_line,
        )
    values = params_mod.check(schema, request.get("params", {}), file, request.get("params_source", "params"))
    for p in schema:
        if p.type == "color":
            code = ctx.color(values[p.name], file, p.line)
            params_mod.check_produced(p, values[p.name], code, ctx, file)
    doc = (fn.__doc__ or "").strip().splitlines()
    title = doc[0] if doc else ""
    if request.get("mode") == "schema":
        params = [{**p.to_json(), "knob": p.name in knobs} for p in schema]
        data = {"model": fn.__name__, "doc": title, "file": file, "params": params}
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
    if ctx.missing or ctx.missing_helpers:
        return _need(ctx)
    ms = int((time.perf_counter() - started) * 1000)
    try:
        summary = write_all(request["out"], fn.__name__, title, result, request.get("manifest", {}), ms)
    except OSError as e:
        raise EmmetError(
            "IO_ERROR",
            f"Can't write the model to {request['out']}/: {e.strerror or e}.",
            "Check that the project folder is writable and the disk isn't full, then run again.",
            e.filename and ctx.rel(e.filename),
            None,
        ) from e
    return {"ok": True, "data": summary}


if __name__ == "__main__":
    sys.exit(main())
