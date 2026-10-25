# SPDX-License-Identifier: Apache-2.0
"""Writing build/model.mpd, build/placements.json, and build/manifest.json (plan section 07). Every write is atomic:
a temp file in the same directory, fsync, rename."""

from __future__ import annotations

import json
import os
import tempfile

from . import __version__
from .core import Model
from .frame import clean, num


def write_atomic(path: str, text: str) -> None:
    folder = os.path.dirname(path) or "."
    os.makedirs(folder, exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=folder, prefix="." + os.path.basename(path) + ".", suffix=".tmp")
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, path)
    except BaseException:
        with _ignore():
            os.unlink(tmp)
        raise


class _ignore:
    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return True


def mpd(name: str, title: str, m: Model) -> str:
    """One `0 FILE` with a `0 !EMMET ID … SRC file:line` line before each part and `0 STEP` after each step."""
    out = [f"0 FILE {name}.ldr", f"0 {title or name}", f"0 Name: {name}.ldr", f"0 !EMMET MODEL {name}", ""]
    for step in m.steps():
        if step.title:
            out.append(f"0 !EMMET STEP {step.title}")
        for p in step.parts:
            r = p.rot
            nums = [*p.pos, r[0][0], r[0][1], r[0][2], r[1][0], r[1][1], r[1][2], r[2][0], r[2][1], r[2][2]]
            out.append(f"0 !EMMET ID {p.id} SRC {p.file}:{p.line}")
            out.append(f"1 {p.color} " + " ".join(num(v) for v in nums) + f" {p.part}.dat")
        out.append("0 STEP")
    out.append("0 NOFILE")
    return "\n".join(out) + "\n"


def placements(m: Model) -> list[dict]:
    rows = []
    for step in m.steps():
        for p in step.parts:
            r, t = p.rot, p.pos
            matrix = [r[0][0], r[1][0], r[2][0], 0, r[0][1], r[1][1], r[2][1], 0, r[0][2], r[1][2], r[2][2], 0]
            matrix += [t[0], t[1], t[2], 1]
            rows.append(
                {
                    "id": p.id,
                    "part": p.part,
                    "color": p.color,
                    "matrix": [clean(v) for v in matrix],
                    "step": p.step,
                    "submodel": [],
                    "src": f"{p.file}:{p.line}",
                }
            )
    return rows


def bbox(m: Model) -> dict | None:
    boxes = [p.box for s in m.steps() for p in s.parts]
    if not boxes:
        return None
    lo = [clean(min(b[0][i] for b in boxes)) for i in range(3)]
    hi = [clean(max(b[1][i] for b in boxes)) for i in range(3)]
    return {"min": lo, "max": hi}


def dumps(obj) -> str:
    return json.dumps(obj, indent=2) + "\n"


def write_all(out_dir: str, name: str, title: str, m: Model, manifest_in: dict, ms: int) -> dict:
    """Writes the three files and returns the run summary."""
    steps = m.steps()
    rows = placements(m)
    box = bbox(m)
    summary = {"parts": len(rows), "steps": len(steps), "submodels": 0, "bbox": box}
    manifest = {
        "schema": 1,
        "model": name,
        "inputs_hash": manifest_in.get("inputs_hash"),
        "emmet": manifest_in.get("emmet"),
        "emmet_dsl": __version__,
        "library": manifest_in.get("library"),
        "parts_data": manifest_in.get("parts_data"),
        **summary,
        "run": {"generation": manifest_in.get("generation"), "ms": ms},
    }
    write_atomic(os.path.join(out_dir, "model.mpd"), mpd(name, title, m))
    write_atomic(os.path.join(out_dir, "placements.json"), dumps({"schema": 1, "model": name, "parts": rows}))
    write_atomic(os.path.join(out_dir, "manifest.json"), dumps(manifest))
    summary["used"] = sorted({r["part"] for r in rows})
    return summary
