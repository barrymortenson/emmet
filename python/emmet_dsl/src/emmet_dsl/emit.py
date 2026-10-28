# SPDX-License-Identifier: Apache-2.0
"""Writing build/model.mpd, build/placements.json, build/steps.json, and build/manifest.json (plan sections 07 and
08). Every write is atomic: a temp file in the same directory, fsync, rename."""

from __future__ import annotations

import json
import os
import stat
import tempfile

from . import __version__, frame
from .core import Model, Placed, Submodel
from .errors import EmmetError
from .frame import clean, num


def write_atomic(path: str, text: str) -> None:
    write_atomic_bytes(path, text.encode("utf-8"))


def write_atomic_bytes(path: str, data: bytes) -> None:
    folder = os.path.dirname(path) or "."
    os.makedirs(folder, exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=folder, prefix="." + os.path.basename(path) + ".", suffix=".tmp")
    try:
        if os.path.exists(path):  # a rewritten model.py keeps its permissions; mkstemp's are owner-only
            os.chmod(tmp, stat.S_IMODE(os.stat(path).st_mode))
        with os.fdopen(fd, "wb") as f:
            f.write(data)
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


class Built:
    """The model ready to write: every model's steps numbered once, and the submodels in the order they're first
    placed."""

    def __init__(self, m: Model):
        self.main = m
        self.steps: dict[int, list] = {}
        self.subs: list[Submodel] = []
        self.copies: dict[str, int] = {}  # submodel file → copies in the whole model
        self._visit(m, 1)

    def _visit(self, m: Model, times: int) -> None:
        for step in self.steps_of(m):
            for p in step.parts:
                if p.sub is None:
                    continue
                if p.sub.file not in self.copies:
                    self.subs.append(p.sub)
                    self.copies[p.sub.file] = 0
                self.copies[p.sub.file] += times
                self._visit(p.sub.model, times)

    def steps_of(self, m: Model) -> list:
        if id(m) not in self.steps:
            self.steps[id(m)] = m._numbered_steps()
        return self.steps[id(m)]


def _line(p: Placed) -> str:
    r = p.rot
    nums = [*p.pos, r[0][0], r[0][1], r[0][2], r[1][0], r[1][1], r[1][2], r[2][0], r[2][1], r[2][2]]
    return f"1 {p.color} " + " ".join(num(v) for v in nums) + f" {p.part if p.sub else p.part + '.dat'}"


def _file(built: Built, m: Model, head: list[str]) -> list[str]:
    out = [*head, ""]
    for step in built.steps_of(m):
        if step.title:
            out.append(f"0 !EMMET STEP {step.title}")
        for p in step.parts:
            out.append(f"0 !EMMET ID {p.id} SRC {p.file}:{p.line}")
            out.append(_line(p))
        out.append("0 STEP")
    out.append("0 NOFILE")
    return out


def mpd(name: str, title: str, built: Built) -> str:
    """The main model's `0 FILE`, then one per submodel, each part preceded by `0 !EMMET ID … SRC file:line` and each
    step ended by `0 STEP`. A submodel copy is a reference to its file."""
    out = _file(
        built, built.main, [f"0 FILE {name}.ldr", f"0 {title or name}", f"0 Name: {name}.ldr", f"0 !EMMET MODEL {name}"]
    )
    for sub in built.subs:
        out += _file(
            built,
            sub.model,
            [f"0 FILE {sub.file}", f"0 {sub.name}", f"0 Name: {sub.file}", f"0 !EMMET SUBMODEL {sub.name}"],
        )
    return "\n".join(out) + "\n"


def _matrix(r, t) -> list:
    m = [r[0][0], r[1][0], r[2][0], 0, r[0][1], r[1][1], r[2][1], 0, r[0][2], r[1][2], r[2][2], 0, t[0], t[1], t[2], 1]
    return [clean(v) for v in m]


class Flat:
    """The model flattened to its parts, in LDraw's frame: placements.json's rows, plus the allow and ground lists
    with every submodel copy's ids."""

    def __init__(self, built: Built):
        self.rows: list[dict] = []
        self.boxes: list[tuple] = []
        self.allow: list[dict] = []
        self.ground: list[str] = []
        identity = ((1, 0, 0), (0, 1, 0), (0, 0, 1))
        self._walk(built, built.main, "", [], [], identity, (0, 0, 0))

    def _walk(self, built: Built, m: Model, prefix: str, path: list, copies: list, outer_r, outer_pos) -> None:
        first = len(self.rows)
        for step in built.steps_of(m):
            for p in step.parts:
                r, pos = frame.compose(outer_r, outer_pos, p.rot, p.pos)
                pid = prefix + p.id
                if p.sub is not None:
                    copy = {"id": pid, "step": p.step}
                    self._walk(built, p.sub.model, pid + "/", [*path, p.sub.file], [*copies, copy], r, pos)
                    continue
                row = {
                    "id": pid,
                    "part": p.part,
                    "color": p.color,
                    "matrix": _matrix(r, pos),
                    "step": p.step,
                    "submodel": path,
                }
                if copies:
                    row["copies"] = copies
                row["src"] = f"{p.file}:{p.line}"
                self.rows.append(row)
                self.boxes.append(frame.box_in_dsl(frame.transform_box(r, pos, p.body)))
        mine = [row["id"] for row in self.rows[first:]]
        for rule, ids, reason, file, line in m._allowed:
            self.allow.append({"rule": rule, "parts": _expand(m, ids, prefix, mine, file, line), "reason": reason})
        for ids, file, line in m._grounded:
            self.ground += _expand(m, ids, prefix, mine, file, line)


def _expand(m: Model, ids: list[str], prefix: str, mine: list[str], file: str, line: int) -> list[str]:
    """Ids as written in a model (a part's, or a submodel copy's) as the part ids of this copy of the model."""
    out = []
    for i in ids:
        if i not in m._ids:
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"There's no part '{i}' to allow or ground.",
                "Use the id of a part or a submodel copy this model placed (a Handle's .id).",
                file,
                line,
            )
        full = prefix + i
        if m._ids[i].sub is None:
            out.append(full)
        else:
            out += [r for r in mine if r.startswith(full + "/")]
    return out


def bbox(flat: Flat) -> dict | None:
    if not flat.boxes:
        return None
    lo = [clean(min(b[0][i] for b in flat.boxes)) for i in range(3)]
    hi = [clean(max(b[1][i] for b in flat.boxes)) for i in range(3)]
    return {"min": lo, "max": hi}


def _volume(p: Placed) -> float:
    b = p.body
    return (b[3] - b[0]) * (b[4] - b[1]) * (b[5] - b[2])


def _step_json(built: Built, step, n: int) -> dict:
    """One step: its new parts (the parts box: part, color, count, larger parts first) and the sub-assemblies it
    places."""
    box: dict[tuple, list] = {}
    subs: dict[str, dict] = {}
    ids = []
    for p in step.parts:
        if p.sub is None:
            ids.append(p.id)
            entry = box.setdefault((p.part, p.color), [0, _volume(p)])
            entry[0] += 1
        else:
            s = subs.setdefault(p.sub.file, {"name": p.sub.name, "file": p.sub.file, "count": 0, "ids": []})
            s["count"] += 1
            s["ids"].append(p.id)
    parts = sorted(box.items(), key=lambda kv: (-kv[1][1], -kv[1][0], kv[0][0], kv[0][1]))
    title = step.title
    return {
        "n": n,
        "title": title,
        "group": title.split(":", 1)[0].strip() if title else "",
        "part_ids": ids,
        "parts": [{"part": part, "color": color, "count": c} for (part, color), (c, _) in parts],
        "submodels": list(subs.values()),
    }


def steps_doc(name: str, built: Built) -> dict:
    """build/steps.json: the main model's steps, then each sub-assembly's own steps and how many copies the model
    has ("× 4")."""
    return {
        "schema": 1,
        "model": name,
        "steps": [_step_json(built, s, n) for n, s in enumerate(built.steps_of(built.main), 1)],
        "submodels": [
            {
                "name": sub.name,
                "file": sub.file,
                "count": built.copies[sub.file],
                "parts": sub.size,
                "steps": [_step_json(built, s, n) for n, s in enumerate(built.steps_of(sub.model), 1)],
            }
            for sub in built.subs
        ],
    }


def dumps(obj) -> str:
    return json.dumps(obj, indent=2) + "\n"


def write_all(out_dir: str, name: str, title: str, m: Model, manifest_in: dict, ms: int) -> dict:
    """Writes the files and returns the run summary."""
    built = Built(m)
    flat = Flat(built)
    steps = built.steps_of(m)
    box = bbox(flat)
    summary = {"parts": len(flat.rows), "steps": len(steps), "submodels": len(built.subs), "bbox": box}
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
    placements = {"schema": 1, "model": name, "parts": flat.rows}
    if flat.allow:
        placements["allow"] = flat.allow
    if flat.ground:
        placements["ground"] = flat.ground
    write_atomic(os.path.join(out_dir, "model.mpd"), mpd(name, title, built))
    write_atomic(os.path.join(out_dir, "placements.json"), dumps(placements))
    write_atomic(os.path.join(out_dir, "steps.json"), dumps(steps_doc(name, built)))
    write_atomic(os.path.join(out_dir, "manifest.json"), dumps(manifest))
    summary["used"] = sorted({r["part"] for r in flat.rows})
    return summary
