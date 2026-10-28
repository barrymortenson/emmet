# SPDX-License-Identifier: Apache-2.0
"""Part helpers: Brick(2, 4), Plate(1, 6), Tile(2, 2), Slope(45, 2, 1) (plan section 08). Each resolves to an LDraw
part id through emmet's parts index, which emmet looks up between passes like any other part; a size the library
doesn't have fails with the sizes it does."""

from __future__ import annotations

from . import core
from .errors import EmmetError, caller

# The piece lengths a run is split into, when the library has them.
LENGTHS = (1, 2, 3, 4, 6, 8, 10, 12, 14, 16)


def _lookup(kind: str, dims: tuple, soft: bool):
    file, line = caller()
    ctx = core.active
    if ctx is None:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            "Part helpers work only inside emmet run.",
            "Run the model with emmet run; it looks the parts up in the library.",
            file,
            line,
        )
    return ctx.helper(kind, dims, soft, ctx.rel(file), line)


class Family:
    """A family of parts by size: Brick, Plate, or Tile. Call it with a size for one part's id (Brick(2, 4) is
    "3001"); pass the family itself to Model.row, Model.fill, or Model.wall."""

    def __init__(self, kind: str, height: int):
        self.kind = kind
        self.height = height  # in plates

    def __call__(self, w: int, l: int, h: int = 1) -> str:  # noqa: E741 - the l of "w x l"
        """The part id of a w × l piece (and h bricks tall, for Brick)."""
        w, l = sorted((int(w), int(l)))  # noqa: E741
        return _lookup(self.kind, (w, l, int(h)), soft=False)

    def piece(self, w: int, n: int) -> str | None:
        """A w × n piece's id, or None when the library has no such size."""
        lo, hi = sorted((int(w), int(n)))
        return _lookup(self.kind, (lo, hi, 1), soft=True)

    def has_width(self, w: int) -> bool:
        return self.piece(w, w) is not None

    def lengths(self, width: int, sizes=None) -> dict[int, str]:
        """{length along the run: part id} for runs `width` studs across: pieces at least as long as the width, and
        shorter ones laid across the run."""
        out = {}
        for n in sizes or LENGTHS:
            part = self.piece(width, n)
            if part is not None:
                out[int(n)] = part
        return out

    def longest(self, run: int, width: int, sizes=None) -> int:
        fits = [n for n in self.lengths(width, sizes) if n <= run]
        return max(fits, default=1)

    def split(self, length: int, width: int, sizes, first, rel: str, line: int) -> list[tuple[int, str]]:
        """A run of `length` as [(length along, part id)], longest first, after an optional first piece."""
        have = self.lengths(width, sizes)
        if not have:
            raise EmmetError(
                "UNKNOWN_PART", f"The library has no {self.kind} {width} wide.", "Use a width of 1 or 2.", rel, line
            )
        out, left = [], int(length)
        if first and int(first) in have and int(first) <= left:
            out.append((int(first), have[int(first)]))
            left -= int(first)
        for n in sorted(have, reverse=True):
            while left >= n:
                out.append((n, have[n]))
                left -= n
        if left:
            raise EmmetError(
                "UNKNOWN_PART",
                f"A run of {length} can't be made from {self.kind} pieces of "
                f"{', '.join(str(n) for n in sorted(have))}.",
                "Use other sizes, or another length.",
                rel,
                line,
            )
        return out

    def __repr__(self) -> str:
        return self.kind.title()


Brick = Family("brick", 3)
Plate = Family("plate", 1)
Tile = Family("tile", 1)


def Slope(angle: int, w: int, l: int) -> str:  # noqa: E741, N802 - named like the other helpers
    """The part id of a slope: Slope(45, 2, 1) is the Slope Brick 45 2 x 1, w deep down the slope and l wide."""
    return _lookup("slope", (int(w), int(l), int(angle)), soft=False)
