# SPDX-License-Identifier: Apache-2.0
"""Technic spine: a long, stiff beam of two rows of Technic bricks side by side, their joints offset, pinned together
through their holes, so it can't bend at a joint.

m.use("technic_spine", x, y, z, length=16, color="Light Bluish Gray", title="Spine") lays two rows of Technic bricks 1 x
8 (3702) and 1 x 4 (3701) along x from (x, y, z) and (x, y + 1, z), `length` studs, the second row starting with a 1 x 4
so its joints fall between the first row's, with a Technic pin pushed into the first row's holes either side of every
joint before the second row presses on over them. Lengths are multiples of 4 from 8. Returns the z of its top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 60440. Not derived from any model file."""

PARTS = ["3702", "3701", "2780"]

PIECES = {8: "3702", 4: "3701"}


def _row(length, first):
    out, left = ([first] if first else []), length - (first or 0)
    while left >= 8:
        out.append(8)
        left -= 8
    if left:
        out.append(left)
    return out


def build(m, x, y, z, length=16, color="Light Bluish Gray", title="Spine"):
    """The two rows and their pins; returns the z of the top."""
    if length % 4 or length < 8:
        raise ValueError(
            f"technic_spine takes a length of 8 or more in steps of 4, not {length}"
        )
    rows = [_row(length, first) for first in (None, 4)]
    joints = set()
    for sizes in rows:
        a = x
        for n in sizes[:-1]:
            a += n
            joints.add(a)

    def lay(k):
        with m.step(f"{title}, row {k + 1}"):
            a = x
            for n in rows[k]:
                m.add(PIECES[n], color, at=(a, y + k, z), rot=1)
                a += n

    lay(0)
    with m.step(f"{title}: pins"):
        # A hole sits between each pair of studs; pin the holes either side of every joint, into the first row, so
        # the second row presses on over them.
        for j in sorted(joints):
            for hx in (j - 1, j + 1):
                m.add("2780", "Black", at=(hx - 0.4, y, z + 0.775), rot=0)
    lay(1)
    return z + 3


def example(m):
    """A spine 16 studs long."""
    m.use("technic_spine", x=0, y=0, z=0)
