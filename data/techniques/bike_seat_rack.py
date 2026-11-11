# SPDX-License-Identifier: Apache-2.0
"""Bike seat and rack: a display e-bike's or motorbike's long flat seat and rear rack on a spine of Technic bricks
pinned along the frame's level top tubes, with tail lights and a rear fender over the wheel.

m.use("bike_seat_rack", frame=frame, seat=11, rack=6, color="Black", rack_color="Dark Bluish Gray", spine_color="Black",
light_color="Trans-Red", fender=True, title="Seat") takes the frame `motorbike_frame` returned. The spine is two
Technic bricks 1 x 16 with holes (3703) side by side between the top tubes, from a stud short of the head back past the
seat tube, each pinned (friction pins, 2780) to its top tube's holes; Technic bricks 1 x 6 (3894) carry it `rack` studs
further back (4 to 8). A layer of plates 6 wide on the spine is the seat's and the rack's base. The seat, `seat` studs
long (6 to 13) from the base's back end forward, is another layer of plates, tiles down the middle, and a curved slope
2 x 1 (11477) across each side so its edges are round; tiles cover the base and the spine in front of it, to the head.
The rack is two rails of 1-wide plates and two rows of tiles between them, slats with gaps, and a 1 x 1 round tile in
each gap at its back end for the tail lights. With `fender`, plates 4 wide under the rack bricks are the rear fender
over the tire; they and the rack bricks go in from below, so they're one section, built on its own and put on whole. It
brings its own steps and returns a dict of "spine_bottom" (the spine's underside, in plates), "grid" (the x the spine's
studs count from: its back end), "seat_top" (in plates), and "spine" (the two spine bricks' Handles), which
`battery_case` uses.
"""

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of motorbikes and Barry's note that
the e-bike he builds has a long flat seat and a rear rack. Not derived from any model file."""

PARTS = [
    "3028",
    "3033",
    "3036",
    "3958",
    "3703",
    "3894",
    "3701",
    "3702",
    "2780",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "3024",
    "3023",
    "3623",
    "3710",
    "3666",
    "3460",
    "3070b",
    "3069b",
    "63864",
    "2431",
    "6636",
    "4162",
    "11477",
    "98138",
]

PLATES2 = {10: "3832", 8: "3034", 6: "3795", 4: "3020", 3: "3021", 2: "3022"}
PLATES1 = {8: "3460", 6: "3666", 4: "3710", 3: "3623", 2: "3023", 1: "3024"}
TILES1 = {8: "4162", 6: "6636", 4: "2431", 3: "63864", 2: "3069b", 1: "3070b"}
PLATES6 = {12: "3028", 10: "3033", 8: "3036", 6: "3958"}
RACK = {4: "3701", 6: "3894", 8: "3702"}
STUD = 0.4  # a plate's height, in studs


def split(n, sizes, offset=0):
    """Lengths from `sizes` that add up to n; with an offset, the first is a short one, so rows don't line up."""
    out, left = [], n
    if offset % 2 and n > 4:
        first = min(k for k in sizes if k >= 2)
        out.append(first)
        left -= first
    while left > 0:
        k = max(k for k in sizes if k <= left)
        out.append(k)
        left -= k
    return out


def row(m, sizes, color, x, y, z, length, offset=0):
    """One row of parts along x from (x, y, z) (studs, studs, plates), 1 or 2 wide by the table."""
    for k in split(length, sizes, offset):
        m.add(sizes[k], color, at=(x, y, z), rot=1)
        x += k


def slab(m, color, x, y, z, length):
    """A layer 6 wide along x from (x, y, z): 6-wide plates, and 1 x 6 plates across for what's left."""
    while length > 0:
        k = max((k for k in PLATES6 if k <= length), default=None)
        if k is None:
            m.add("3666", color, at=(x, y, z))
            x, length = x + 1, length - 1
        else:
            m.add(PLATES6[k], color, at=(x, y, z), rot=1)
            x, length = x + k, length - k


def build(
    m,
    frame,
    seat=11,
    rack=6,
    color="Black",
    rack_color="Dark Bluish Gray",
    spine_color="Black",
    light_color="Trans-Red",
    fender=True,
    title="Seat",
):
    """The spine, the base, the seat, the rack and its lights, and the rear fender; returns where the spine is."""
    if not 6 <= seat <= 13:
        raise ValueError(f"bike_seat_rack: seat {seat} isn't 6 to 13 studs")
    if rack not in RACK:
        raise ValueError(f"bike_seat_rack: rack {rack} isn't 4, 6, or 8 studs")
    to_model, y = frame["to_model"], frame["y"]
    H = frame["nodes"]["head"]
    top_len, first = frame["lengths"]["top"], frame["firsts"]["top"]
    hx, _, hz = to_model(*H)
    grid = hx - 17  # the spine's back end: it ends a stud short of the head
    # The spine's top, and the top tubes' (a Technic brick's hole is 1.75 plates up its 3)
    top = hz + 2.5 * 0.5
    bottom = top - 3
    with_pins = [j for j in (2, 5, 8, 11) if j < top_len]
    spine = []
    for s, name in ((-1, "right"), (1, "left")):
        with m.step(f"{title}: spine, {name} side"):
            spine.append(
                m.add(
                    "3703", spine_color, at=(grid + 6, y + s * 0.5, hz), rot=3, hole="6"
                )
            )
            for j in with_pins:
                hole = frame["sides"][name]["top"].hole(first + j)
                m.insert("2780", "Black", hole, out="+y" if s < 0 else "-y", out_by=1)
    base = 19 - 6 + rack  # from the rack's back end to a stud past the seat's front
    with m.step(f"{title}: base"):
        slab(m, color, grid - rack, y - 3, top, base)

    def underside(mm):
        for s in (0, 1):
            mm.add(RACK[rack], spine_color, at=(0, s, 3 * fender), rot=3)
        if fender:
            for r in (0, 1):
                row(mm, PLATES2, color, 0, -1 + 2 * r, 2, rack, offset=r)

    with m.step(f"{title}: rack's underside" + (" and rear fender" if fender else "")):
        m.place(
            m.submodel(f"seat-rack-{rack}-{int(fender)}", underside),
            at=(grid - rack, y - (2 if fender else 1), bottom - (1 if fender else 0)),
        )
    z1, z2 = top + 1, top + 2
    with m.step(f"{title}: seat"):
        slab(m, color, grid + 1, y - 3, z1, seat)
        row(m, TILES1, color, grid + 1, y - 1, z2, seat)
        row(m, TILES1, color, grid + 1, y, z2, seat, offset=1)
    for y0, name in ((-3, "right"), (1, "left")):
        for part in range(2):
            ks = [k for k in range(seat) if k % 2 == part]
            with m.step(
                f"{title}: {name} edge"
                + (", back half" if part == 0 else ", front half")
            ):
                for k in ks:
                    m.add(
                        "11477",
                        color,
                        at=(grid + 1 + k, y + y0, z2),
                        rot=1 if y0 > 0 else 3,
                    )
    front = grid + 1 + seat  # the seat's front end
    with m.step(f"{title}: in front of it"):
        left = grid + 13 - front
        for r, y0 in enumerate((-3, -2, -1, 0, 1, 2)):
            if left > 0:
                row(m, TILES1, color, front, y + y0, z1, left, offset=r)
        for y0 in (-1, 0):
            row(m, TILES1, spine_color, grid + 13, y + y0, top, 3)
    with m.step(f"{title}: rack"):
        for y0 in (-3, 2):  # the rails
            row(m, PLATES1, color, grid - rack, y + y0, z1, rack + 1)
        for y0 in (-2, 0):  # the slats, with gaps between
            row(m, TILES1, rack_color, grid - rack, y + y0, z1, rack)
        for y0 in (-1, 1):  # the tail lights, at the back end of the gaps
            m.add("98138", light_color, at=(grid - rack, y + y0, z1))
    return {
        "spine_bottom": bottom,
        "grid": grid,
        "seat_top": z2 + 1,
        "spine": spine,
        "top": top,
    }


def example(m):
    """A seat and rack on a frame with its swingarm and wheel."""
    frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
    m.use("swingarm_shock", frame=frame)
    m.use("bike_seat_rack", frame=frame)
