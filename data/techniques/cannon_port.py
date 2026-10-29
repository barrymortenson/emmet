# SPDX-License-Identifier: Apache-2.0
"""Cannon port: a gun port in a hull side with a lid on a bar-and-clip hinge along its sill, shown let down open.

m.use("cannon_port", x, y, z, length, ports=None, rows=3, color="Dark Red", lid="Black", title="Gun ports") builds a
hull side 1 stud thick and `length` studs long along x from (x, y), `rows` bricks tall from z, its face to -y, with a
port 3 studs wide in its second course at each of `ports` (studs from the side's start; by default one in the middle).
On each port's sill, at its two outer studs, plates 1 x 1 with a clip (4081b) point their clips out; each grips the
handle of a plate 1 x 2 with a handle (2540), centered on it, so the two handle plates sit half a stud off the hull's
grid, and a plate 1 x 4 across them makes the lid, 4 studs wide, which lies open level with the clips, out from the
hull; turned up about the handles it closes the port, lapping half a stud of hull each side. The port is two plates high
over the clips, and the course over it spans it. Returns the z of the side's top.
"""

from emmet_dsl import Brick

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "4081b",
    "2540",
    "3710",
    "3005",
    "3004",
    "3622",
    "3010",
    "3009",
    "3008",
    "6111",
    "6112",
    "4217",
    "2465",
]

CLIP_OUT = 0.9  # how far a clip plate's clip reaches past its body
LID_OUT = 2  # how far out from the hull's face the open lid's outer edge lies; its handles reach back into the clips


def build(
    m,
    x,
    y,
    z,
    length,
    ports=None,
    rows=3,
    color="Dark Red",
    lid="Black",
    title="Gun ports",
):
    """The side a course at a time, with the ports, their clips and lids; returns the z of the side's top."""
    ports = [(length - 3) // 2] if ports is None else sorted(ports)
    if rows < 3:
        raise ValueError(f"cannon_port needs 3 rows or more, not {rows}")
    if (
        ports[0] < 1
        or ports[-1] > length - 4
        or any(ports[i + 1] - ports[i] < 5 for i in range(len(ports) - 1))
    ):
        raise ValueError(
            f"cannon_port needs its ports inside the side, 2 studs of side between, not at {ports}"
        )
    for r in range(rows):
        zr = z + 3 * r
        with m.step(f"{title}, row {r + 1}"):
            gaps = [(p, p + 3) for p in ports] if r == 1 else []
            u = 0
            for start, end in gaps + [(length, length)]:
                n = start - u
                if n > 0:
                    first = max(1, n // 2) if r % 2 == 0 and r and n > 2 else None
                    m.row(Brick, color, at=(x + u, y, zr), length=n, first=first)
                u = end
        if r == 0:
            for p in ports:
                with m.step(f"{title}: port at {p}"):
                    for k in (0, 2):
                        m.add(
                            "4081b", lid, at=(x + p + k, y - CLIP_OUT, z + 2.75), rot=3
                        )
                        m.add(
                            "2540", lid, at=(x + p + k - 0.5, y - LID_OUT, z + 3), rot=1
                        )
                    m.add("3710", lid, at=(x + p - 0.5, y - LID_OUT, z + 4), rot=1)
    return z + 3 * rows


def example(m):
    """A stretch of hull side with one gun port, its lid open."""
    m.use("cannon_port", x=0, y=0, z=0, length=7)
