# SPDX-License-Identifier: Apache-2.0
"""Motorbike frame: a display motorbike's or e-bike's frame of Technic beams on edge, fitted to the bike in the photo in
whole studs so every joint is a pin or an axle in a hole, with a level top tube for the seat and battery to hang on.

m.use("motorbike_frame", x, y, z, head=(9.8, 5.4), crank=(3.3, -4.1), seat=(-2.0, 4.6), rear=(-8.2, -3.6), rake=24,
color="Dark Blue", pin_color="Black", axle_color="Black", lock_color=None, title="Frame") builds it round the
swingarm's pivot at (x, y, z), the bike running along +x across the center plane y. Measure the photo first, each point
(run, rise) in studs from the pivot, the wheels as the ruler: `head` where the top tube and the down tube meet the
steering head, `crank` the bottom bracket, `seat` the top tube's back end (the top of the seat tube), and `rear` the rear
axle (it sets the swingarm's angle); `rake` is the steering axis's lean from upright, in degrees. The defaults are a
fat-tire e-bike's.

Each side has five beams: a top tube from the head back to the seat tube's top, a seat tube from there down to the
pivot, a down tube from the head to the crank, a link from the crank back to the pivot, and a rail from the pivot
forward to the down tube, which braces the frame under the battery. Their lengths are the whole studs (3 to 14) that put
the joints nearest the photo's, and the frame turns about the pivot so the top tube is level: `bike_seat_rack` pins a
spine of Technic bricks to its holes, and `battery_case` hangs under it. A beam whose length is odd has a spare hole,
where nothing else is: below the crank, behind the pivot, past the rail's front joint, or behind the seat tube.

From the outside in, a side's planes are the rail's, then the seat tube's and down tube's, then the top tube's and the
link's; the swingarm goes outside them. The right side (-y) goes together first, then the cross axles through it: the
pivot's, an axle 12 (3708), a stud out past each swingarm arm for its bushes, and the crank's, an axle 10 (3737), for
`chain_drive`'s crank arms. Then the left side goes on the axles. Last, the head: a thin liftarm (6632) each side,
pinned to the down tube's next hole, and a Technic cross block 1 x 2 (6536) between the sides whose pin hole, a stud
forward, runs along the steering axis, then the head's axle 7 (44294) pushed in through them all, so nothing goes onto
an axle after both its ends are closed; the liftarms hold the axle in and stop it turning, so the bearing on it can't
tip. The fork's steerer turns in it (`suspension_fork`). The rear shock's top goes on an axle 6 through a down tube hole
near the crank, chosen with the swingarm's angle so a shock absorber 6.5L reaches it from a stud behind the pivot:
`swingarm_shock` pushes that axle in through the shock.

It brings its own steps and returns the frame: a dict of "pivot", "y", "steering" (the bearing's pin hole) and "head"
(its middle), "rake" (a stud down the steering axis, as run and rise) and "forward" (a stud square to it, forward),
"arm_angle" (the swingarm's, in degrees below level), "shock_mount" (the shock axle's middle) and "shock_mount_hole"
(the right down tube's hole it goes in), "crank" (the crank
axle's middle), "nodes" (the joints, (run, rise) in studs from the pivot: "pivot", "crank", "rail", "head", "seat"),
"to_model" (a function from (run, rise, across) studs to a point in the model), "lengths", "planes", and "sides" ("right"
and "left", each a dict of the "top", "seat", "down", "link", and "rail" beams' Handles), which `swingarm_shock`,
`suspension_fork`, `chain_drive`, `bike_seat_rack`, and `battery_case` use.
"""

import functools
import math

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic frames and Barry's notes
on his e-bike hand sessions: the axles need something to retain them, and the head mustn't tip. Not derived from any
model file."""

PARTS = [
    "32523",
    "32316",
    "32524",
    "40490",
    "32525",
    "41239",
    "32278",
    "2780",
    "44294",
    "3708",
    "3737",
    "6632",
    "6536",
]

BEAMS = {
    3: "32523",
    5: "32316",
    7: "32524",
    9: "40490",
    11: "32525",
    13: "41239",
    15: "32278",
}
AXLES = {7: "44294", 10: "3737", 12: "3708"}
# The planes' middles, studs from the center plane on each side.
PLANES = {
    "top": 1.5,
    "link": 1.5,
    "seat": 2.5,
    "down": 2.5,
    "rail": 3.5,
    "lock": 3.25,
    "arm": 4.5,
}
SHOCK = 5.5  # studs between a shock absorber 6.5L's holes
# Where the search starts the link; the frame is turned level after
TILT = math.radians(-47)


def _circle(a, ra, b, rb):
    """Where a circle of radius ra round a meets one of rb round b: the meeting below the line from a to b."""
    dx, dz = b[0] - a[0], b[1] - a[1]
    d = math.hypot(dx, dz)
    if d == 0 or d > ra + rb or d < abs(ra - rb):
        return None
    along = (ra * ra - rb * rb + d * d) / (2 * d)
    h = math.sqrt(max(ra * ra - along * along, 0.0))
    ux, uz = dx / d, dz / d
    return (a[0] + along * ux + h * uz, a[1] + along * uz - h * ux)


def _clear(p, a, b):
    """How far p is from the segment a-b."""
    dx, dz = b[0] - a[0], b[1] - a[1]
    t = max(
        0.0, min(1.0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dz) / (dx * dx + dz * dz))
    )
    return math.dist(p, (a[0] + t * dx, a[1] + t * dz))


def _arm_angle(mount):
    """The swingarm's angle below level (radians) that puts its hole a stud behind the pivot SHOCK studs from `mount`,
    or None."""
    rho = math.hypot(*mount)
    q = (SHOCK * SHOCK - 1 - rho * rho) / 2
    # |S - mount|² = 1 + rho² - 2 S . mount = SHOCK², with S = (-cos, -sin): mount . (cos, sin) = q.
    c = q / rho
    if abs(c) > 1:
        return None
    delta = math.atan2(mount[1], mount[0])
    for th in (delta + math.acos(c), delta - math.acos(c)):
        th = (th + math.pi) % (2 * math.pi) - math.pi
        if math.radians(5) <= th <= math.radians(45):
            return th
    return None


@functools.lru_cache(maxsize=32)
def fit(head, crank, seat, rear):
    """The whole-stud frame nearest the photo's joints: (link, down, rail_at, rail, top, seat_tube, mount) lengths in
    studs (mount: the shock axle's hole, studs up the down tube from the crank), the joints after leveling, and the
    swingarm's angle."""
    target = math.atan2(-rear[1], -rear[0])
    best = None
    for c in range(3, 8):
        b0 = (c * math.cos(TILT), c * math.sin(TILT))
        for d in range(8, 15):
            for k in range(2, d - 1):
                for r in range(3, 12):
                    x0 = _circle(b0, k, (0.0, 0.0), r)
                    if x0 is None:
                        continue
                    u = ((x0[0] - b0[0]) / k, (x0[1] - b0[1]) / k)
                    h0 = (b0[0] + d * u[0], b0[1] + d * u[1])
                    for a in range(8, 15):
                        for s in range(3, 9):
                            t0 = _circle(h0, a, (0.0, 0.0), s)
                            if t0 is None:
                                continue
                            turn = -math.atan2(h0[1] - t0[1], h0[0] - t0[0])
                            cs, sn = math.cos(turn), math.sin(turn)

                            def rot(p, cs=cs, sn=sn):
                                return (p[0] * cs - p[1] * sn, p[0] * sn + p[1] * cs)

                            B, X, H, T = rot(b0), rot(x0), rot(h0), rot(t0)
                            if not (T[0] < 0 < B[0] < X[0] < H[0]):
                                continue
                            err = (
                                math.dist(B, crank)
                                + math.dist(H, head)
                                + math.dist(T, seat)
                            )
                            if best is not None and err > best[0]:
                                continue
                            v = ((X[0] - B[0]) / k, (X[1] - B[1]) / k)
                            for kd in (1, 2, 3):
                                mount = (B[0] + kd * v[0], B[1] + kd * v[1])
                                th = _arm_angle(mount)
                                if th is None:
                                    continue
                                low = (-math.cos(th), -math.sin(th))
                                # The shock's body clears the pivot and crank axles.
                                if (
                                    min(
                                        _clear((0.0, 0.0), low, mount),
                                        _clear(B, low, mount),
                                    )
                                    < 0.8
                                ):
                                    continue
                                spares = sum(n % 2 for n in (c, d, r, a, s))
                                # The rail braces the battery's bottom, so it rises from the pivot as in a photo.
                                score = (
                                    err
                                    + 6 * abs(th - target)
                                    + 0.05 * spares
                                    + 0.3 * max(0.0, -X[1])
                                )
                                if best is None or score < best[0]:
                                    best = (
                                        score,
                                        (c, d, k, r, a, s, kd),
                                        (B, X, H, T),
                                        th,
                                    )
    if best is None:
        raise ValueError(
            "motorbike_frame: no whole-stud frame fits these points; check that head, crank, seat, and rear are "
            "(run, rise) studs from the pivot"
        )
    return best[1:]


def _beam(n):
    """The beam for a span of n studs: n + 1 holes, or n + 2 with a spare."""
    return BEAMS[n + 1] if n % 2 == 0 else BEAMS[n + 2]


def build(
    m,
    x,
    y,
    z,
    head=(9.8, 5.4),
    crank=(3.3, -4.1),
    seat=(-2.0, 4.6),
    rear=(-8.2, -3.6),
    rake=24,
    color="Dark Blue",
    pin_color="Black",
    axle_color="Black",
    lock_color=None,
    title="Frame",
):
    """The two sides, the cross axles, the head locks, and the bearing; returns the frame (see the module's doc)."""
    lengths, (B, X, H, T), th = fit(tuple(head), tuple(crank), tuple(seat), tuple(rear))
    c, d, k, r, a, s, kd = lengths
    P = (0.0, 0.0)

    def to_model(run, rise, across=0.0):
        return (x + run, y + across, z + 2.5 * rise)

    def beam(n, p, q, plane, first):
        """A beam of span n on edge from p to q (studs from the pivot), hole `first` on p."""
        return m.beam(
            _beam(n),
            color,
            at=to_model(*p, plane),
            to=to_model(*q, plane),
            holes=(first, first + n),
        )

    # Where each beam's spare hole goes, if it has one: below the crank, behind the pivot, past X, behind T.
    firsts = {"down": 1 + d % 2, "link": 1 + c % 2, "rail": 1, "top": 1, "seat": 1}
    sides = {}

    def side(sgn, name):
        out = "+y" if sgn < 0 else "-y"
        with m.step(f"{title}, {name} side"):
            pl = {key: sgn * v for key, v in PLANES.items()}
            parts = {
                "down": beam(d, B, H, pl["down"], firsts["down"]),
                "seat": beam(s, T, P, pl["seat"], firsts["seat"]),
                "top": beam(a, H, T, pl["top"], firsts["top"]),
                "link": beam(c, P, B, pl["link"], firsts["link"]),
                "rail": beam(r, P, X, pl["rail"], firsts["rail"]),
            }
            # T: the seat tube into the top tube; X: the rail into the down tube.
            m.insert("2780", pin_color, parts["seat"].hole(firsts["seat"]), out=out)
            m.insert("2780", pin_color, parts["rail"].hole(firsts["rail"] + r), out=out)
        sides[name] = parts

    side(-1, "right")
    right = sides["right"]
    mount_hole = right["down"].hole(firsts["down"] + kd)

    def across(n, hole, hi):
        """An axle n long through `hole` on the right side, its far end hi studs from the center plane."""
        face = hole.at[1] + hole.length / 2
        return m.insert(AXLES[n], axle_color, hole, out="+y", out_by=y + hi - face)

    with m.step(f"{title}: cross axles"):
        # The pivot, out to the swingarm's bushes
        across(12, right["link"].hole(firsts["link"]), 6)
        across(10, right["link"].hole(firsts["link"] + c), 5)  # the crank
    side(1, "left")
    r_ = math.radians(rake)
    forward, down = (math.cos(r_), math.sin(r_)), (math.sin(r_), -math.cos(r_))
    with m.step(f"{title}: head"):
        for sgn, name in ((-1, "right"), (1, "left")):
            # On the head axle's end, pinned to the down tube's next hole: it holds the axle in and stops it turning.
            nxt = sides[name]["down"].hole(firsts["down"] + d - 1)
            lock = m.beam(
                "6632",
                lock_color or color,
                at=to_model(*H, sgn * PLANES["lock"]),
                to=(nxt.at[0], y + sgn * PLANES["lock"], nxt.at[2]),
                axis="y",
                holes=(1, 2),
            )
            m.insert(
                "2780",
                pin_color,
                lock.hole(2),
                out="+y" if sgn < 0 else "-y",
                out_by=1.0,
            )
        bearing = m.beam(
            "6536",
            color,
            at=to_model(*H),
            to=to_model(H[0] + forward[0], H[1] + forward[1]),
            axis="y",
            holes=(2, 1),
        )
        # In last, through the liftarms, the sides, and the bearing: they're all on it when its ends close.
        across(7, right["top"].hole(firsts["top"]), 3.5)
    steering = bearing.hole(1)
    D = (B[0] + kd * (X[0] - B[0]) / k, B[1] + kd * (X[1] - B[1]) / k)
    return {
        "pivot": (x, y, z),
        "y": y,
        "steering": steering,
        "head": steering.at,
        "rake": down,
        "forward": forward,
        "arm_angle": math.degrees(th),
        "shock_mount": to_model(*D),
        "shock_mount_hole": mount_hole,
        "crank": to_model(*B),
        "nodes": {"pivot": P, "crank": B, "rail": X, "head": H, "seat": T},
        "to_model": to_model,
        "lengths": {
            "link": c,
            "down": d,
            "rail_at": k,
            "rail": r,
            "top": a,
            "seat": s,
            "mount": kd,
        },
        "planes": dict(PLANES),
        "sides": sides,
        "firsts": firsts,
    }


def example(m):
    """The frame on its own, fitted to a fat-tire e-bike's photo."""
    m.use("motorbike_frame", x=10, y=6, z=22.5)
