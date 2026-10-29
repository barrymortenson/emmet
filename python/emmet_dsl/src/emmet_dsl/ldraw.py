# SPDX-License-Identifier: Apache-2.0
"""import_ldraw: an existing LDraw model as fixed geometry (plan section 08)."""

from __future__ import annotations

import os

from . import core, frame
from .core import Model, Placed, Submodel, _file_name, _Step
from .errors import EmmetError, caller


def import_ldraw(path: str) -> Submodel:
    """Brings an existing LDraw model (an .ldr or .mpd inside the project) in as fixed geometry: a submodel to place
    with Model.place. Its parts' ids are their line numbers ("L12", or "L4/L12" for line 12 of the submodel that line
    4 places), its `0 STEP` lines are its steps, and findings point at its lines."""
    file, line = caller()
    ctx = core.active
    if ctx is None:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            "import_ldraw ran outside emmet run.",
            "Run the model with emmet run; it supplies the parts.",
            file,
            line,
        )
    rel = ctx.rel(file)
    key = ("import_ldraw", str(path), "")
    if key in ctx.submodels:
        return ctx.submodels[key]
    full = os.path.realpath(os.path.join(ctx.root, str(path)))
    if not full.startswith(os.path.realpath(ctx.root) + os.sep) or not os.path.isfile(full):
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"There's no LDraw file '{path}' in the project.",
            "Put the .ldr or .mpd in the project folder and give its path relative to it.",
            rel,
            line,
        )
    # utf-8-sig drops the byte order mark editors on Windows often save before the first line.
    with open(full, encoding="utf-8-sig", errors="replace") as f:
        files = _split(f.read())
    src = os.path.relpath(full, ctx.root).replace(os.sep, "/")
    m = Model()
    step = _Step("", "l")
    m._steps.append(step)
    identity = ((1, 0, 0), (0, 1, 0), (0, 0, 1))

    def walk(lines, r0, pos0, color0, prefix, depth, top):
        nonlocal step
        for n, text in lines:
            words = text.split()
            if top and words[:2] == ["0", "STEP"]:
                step = _Step("", "l")
                m._steps.append(step)
            if not words or words[0] != "1":
                continue
            try:
                if len(words) < 15:
                    raise ValueError
                color = int(words[1], 0)  # an LDraw code, or a direct color like 0x2FF0000
                nums = [float(v) for v in words[2:14]]
            except ValueError:
                raise EmmetError(
                    "MODEL_EXEC_FAILED",
                    f"{src} line {n} isn't a part line LDraw can read: {text.strip()[:80]}",
                    "Fix that line in the file (1 color x y z a b c d e f g h i part.dat), then run again.",
                    src,
                    n,
                ) from None
            color = color0 if color == 16 else color
            name = " ".join(words[14:]).replace("\\", "/").lower()
            r, pos = frame.compose(r0, pos0, (tuple(nums[3:6]), tuple(nums[6:9]), tuple(nums[9:12])), nums[0:3])
            pid = f"{prefix}L{n}"
            inner = files.get(name)
            # A part embedded in the file (an unofficial or custom part) is one part, as the Rust reader takes it.
            if inner is not None and depth < 16 and not _embedded_part(inner):
                walk(inner, r, pos, color, pid + "/", depth + 1, False)
                continue
            part = name[:-4] if name.endswith(".dat") else name
            body = ctx.body(part, src, n)
            box = frame.box_in_dsl(frame.transform_box(r, pos, body))
            placed = Placed(pid, part, color, pos, r, body, box, src, n, sub=None)
            step.parts.append(placed)
            m._ids[pid] = placed

    first = next(iter(files.values()), [])
    walk(first, identity, (0, 0, 0), 16, "", 0, True)
    if not m._ids:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"'{path}' has no parts.",
            "Import an LDraw model: a file with type 1 lines that place parts.",
            rel,
            line,
        )
    stem = os.path.splitext(os.path.basename(full))[0]
    sub = Submodel(stem, _file_name(ctx, stem), m)
    ctx.submodels[key] = sub
    return sub


PART_TYPES = ("Part", "Subpart", "Primitive", "8_Primitive", "48_Primitive", "Shortcut")


def _embedded_part(lines) -> bool:
    """A file of the document marked `0 !LDRAW_ORG [Unofficial_]Part …`: a part, not a submodel."""
    for _n, text in lines:
        words = text.split()
        if words[:2] == ["0", "!LDRAW_ORG"] and len(words) > 2:
            kind = words[2].removeprefix("Unofficial_")
            return kind in PART_TYPES
    return False


def _split(text: str) -> dict[str, list[tuple[int, str]]]:
    """The document's files by lowercased name, each its (line number, text) lines; a plain .ldr is one file."""
    files: dict[str, list] = {}
    current: list = []
    name = ""
    for n, raw in enumerate(text.splitlines(), 1):
        # A byte order mark before a line (an .mpd pieced together from files a Windows editor saved) isn't text.
        raw = raw.lstrip("\ufeff")
        words = raw.split()
        if words[:2] == ["0", "FILE"]:
            name = " ".join(words[2:]).replace("\\", "/").lower()
            current = files.setdefault(name, [])
            continue
        if words[:2] == ["0", "NOFILE"]:
            continue
        if not files:
            if not words:  # blank lines before the first 0 FILE don't make a file of their own
                continue
            current = files.setdefault(name, [])
        current.append((n, raw))
    return files
