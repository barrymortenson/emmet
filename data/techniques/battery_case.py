# SPDX-License-Identifier: Apache-2.0
"""Battery case: an e-bike's battery, a block of bricks hung under the seat's spine inside the frame's main triangle, as
long at each height as the frame's tubes leave room for.

m.use("battery_case", frame=frame, seat=seat, width=6, color="Black", panel="Dark Bluish Gray", title="Battery") takes
what `motorbike_frame` and `bike_seat_rack` returned. Under the spine, a course of 1 x 6 plates across the bike ties the
case to it, then courses of 2-wide bricks (two of them, the second in `panel`'s color for the case's stripe) and plates,
each running from just inside the seat tube to just inside the down tube at its height, and stopping above the rail,
until the window is too narrow (under 3 studs). The rows run in a bond, so each course holds the one above. The frame
closes round it, so it's one section, built on its own and put on whole. `width` is 2, 4, or 6 studs (even). It brings
its own step and returns how many courses it has.
"""

import math

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of e-bikes and Barry's note that the
e-bike he builds carries its battery in the frame's triangle. Not derived from any model file."""

PARTS = [
    "3666",
    "3710",
    "3023",
    "3007",
    "2456",
    "3001",
    "3002",
    "3003",
    "3832",
    "3034",
    "3795",
    "3020",
    "3021",
    "3022",
]

ACROSS = {6: "3666", 4: "3710", 2: "3023"}
BRICKS2 = {8: "3007", 6: "2456", 4: "3001", 3: "3002", 2: "3003"}
PLATES2 = {10: "3832", 8: "3034", 6: "3795", 4: "3020", 3: "3021", 2: "3022"}


def _split(n, sizes, offset):
    """Lengths from `sizes` that add up to n (every size is 2 or more, so none leaves a single stud); with an odd
    offset, a short one first, so a course's joints don't line up with the one above."""
    out, left = [], n
    if offset % 2 and n >= 4:
        out.append(2)
        left -= 2
    while left > 0:
        k = max(k for k in sizes if k <= left and left - k != 1)
        out.append(k)
        left -= k
    return out


def build(
    m, frame, seat, width=6, color="Black", panel="Dark Bluish Gray", title="Battery"
):
    """The courses, as one section under the spine; returns how many courses."""
    if width not in ACROSS:
        raise ValueError(f"battery_case: width {width} isn't 2, 4, or 6 studs")
    n = frame["nodes"]
    P, B, X, H, T = n["pivot"], n["crank"], n["rail"], n["head"], n["seat"]
    px, y, pz = frame["pivot"]
    # The spine's studs count from here, in studs from the pivot
    grid = seat["grid"] - px

    def on_line(p, q, z):
        return p[0] + (z - p[1]) * (q[0] - p[0]) / (q[1] - p[1])

    def margin(p, q):
        """How far, along x, a brick has to stay from the line p-q to clear its beam (half a stud thick)."""
        return 0.55 / math.sin(math.atan2(abs(q[1] - p[1]), abs(q[0] - p[0])))

    rear_m, front_m = margin(T, P), margin(B, H)
    slope = (X[1] - P[1]) / (X[0] - P[0])
    rail_m = 0.55 / math.cos(math.atan(slope))
    z = (seat["spine_bottom"] - pz) / 2.5  # studs above the pivot
    courses = []
    while True:
        h = 0.4 if not courses else 1.2 if len(courses) < 3 else 0.4
        zt, zb = z, z - h
        rear = max(on_line(T, P, zt), on_line(T, P, zb)) + rear_m
        front = min(on_line(B, H, zt), on_line(B, H, zb)) - front_m
        front = min(front, (zb - rail_m - P[1]) / slope + P[0]) if slope > 0 else front
        if slope <= 0 and zb < rail_m + max(X[1], 0):
            break
        lo, hi = math.ceil(rear - grid - 1e-6), math.floor(front - grid + 1e-6)
        if hi - lo < 3:
            break
        courses.append((lo, hi, zb, h))
        z = zb
    if not courses:
        raise ValueError(
            "battery_case: the frame's main triangle has no room for a battery under the spine"
        )
    first = min(c[0] for c in courses)
    low = courses[-1][2]

    def case(mm):
        for i, (lo, hi, zb, h) in enumerate(courses):
            zp = round((zb - low) * 2.5, 6)
            adds = []
            if i == 0:  # plates across, under the spine: they tie the rows together
                adds = [
                    (ACROSS[width], color, (k - first, 0, zp), 0) for k in range(lo, hi)
                ]
            else:
                parts = BRICKS2 if h > 1 else PLATES2
                for r in range(width // 2):
                    x = lo - first
                    for k in _split(hi - lo, parts, i + r):
                        adds.append(
                            (parts[k], panel if i == 2 else color, (x, 2 * r, zp), 1)
                        )
                        x += k
            chunks = [adds[j : j + 8] for j in range(0, len(adds), 8)]
            for j, chunk in enumerate(chunks):
                with mm.step(
                    f"Course {i + 1}" + (f", part {j + 1}" if len(chunks) > 1 else "")
                ):
                    for part, c, at, rot in chunk:
                        mm.add(part, c, at=at, rot=rot)

    with m.step(title):
        m.place(
            m.submodel(
                f"battery-{width}-" + "-".join(f"{c[0]}.{c[1]}" for c in courses), case
            ),
            at=(px + grid + first, y - width / 2, pz + 2.5 * low),
        )
    return len(courses)


def example(m):
    """A battery under a seat, in a frame with its swingarm and wheel."""
    frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
    m.use("swingarm_shock", frame=frame)
    seat = m.use("bike_seat_rack", frame=frame)
    m.use("battery_case", frame=frame, seat=seat)
