# ADR 0013: The connection model and collision proxies

Status: accepted, 2026-10-28 (session 3A)

## Context

Plan section 11 asks validate v1 for three numbers: no error findings on official models, ≥ 95 % of injected faults
found (≥ 90 % per error rule), and ≤ 300 ms for 500 parts and ≤ 1.5 s for 2,000. v0 (ADR 0010) knew studs only and
tested collisions with body boxes shrunk 1 LDU, which can't see into a part's shape: a slope's empty underside, an arch,
a window's opening. 3A replaces both. Section 06 names parry3d; nothing else was added.

## Decisions

### Connectors: LDCad's snap model, computed per run

- **The model is LDCad's**, not section 11's flat `Connector { kind, pos, dir, gender, radius }`: a cylinder is a run
  of sections (radius, length, round or axle or flexible), plus clips, hinge fingers, and generic plugs, with `center`,
  `grid`, `caps`, `slide`, and `scale` as LDCad defines them. The shadow data is written that way, and flattening it
  would lose what tells a pin's ridge from its hole or a stud from a bar. Rejected: converting it to section 11's struct.
- **Sources, in order:** the primitive table (top studs, underside tubes), the shadow data (D-44), and the overrides
  in `data/connections/*.toml`, each with a test model; an override can `replace` a part's connectors. A file's
  connectors are its subfiles' plus its own, as in LDCad, so a new part built from known subparts is covered.
- **Computed per run**, once per part file and kept for the run, not stored in the index: the shadow data is compiled
  in, so connectors change with Emmet, not with the library, and an index built by an older Emmet can't serve stale
  ones. A 2,000-part model's connectors take a few milliseconds.
- **Matching tolerances** are D-45's: 0.5 LDU across the axis for studs, 1 LDU for round joints.

### Collision proxies: voxel boxes

A part's proxy is its body (the triangles, without top studs and underside tubes and posts) filled as 1 LDU voxels
and merged into boxes in its own frame (`check/proxy.rs`). One vertical ray per whole-LDU column, through the column's
middle; crossings pair up; a voxel is solid when its middle is inside. A brick's faces are on whole LDU, so parts that
only touch never overlap. A column with an odd number of crossings (a gap in the mesh) is left empty, and a part with
more than 2,048 boxes is filled again at a coarser step.

- **Threshold: more than 1 LDU** (D-46), past the voxels' own resolution.
- **Seat exemption:** an overlap within a connected pair's seat (its radius plus 2 LDU, half its length plus 2 LDU)
  isn't a collision; nor is one between parts one holds the other (a frame and its glass, a tyre and its wheel).

Rejected:

- **The triangle mesh itself** (parry3d's `TriMesh` against `TriMesh`). Exact, but a mesh has no inside: two
  overlapping bricks whose faces don't cross (one inside the other) don't collide, and LDraw meshes aren't closed
  (studs and tubes are separate primitives with open ends). Depth needs a closed volume.
- **Convex decomposition** (V-HACD or CoACD). Not named in section 06 (a new dependency), and approximate: its hulls
  round off the very corners and openings that make a slope or arch fit, and a tolerance to hide that hides faults too.
- **Our own box test** (v0's separating axis test, kept for the boxes). It worked for quarter turns but needed its own
  depth and axis for turned boxes. parry3d's contact query gives both, so ours went.

### Built on first use, cached in the index

A proxy is built the first time a model uses the part and stored in the parts index (`proxies`, index schema 3, as
little-endian i16 boxes), so each part is built once per library and validation is warm from then on.

Rejected: building every part's proxy when the index is built. The whole library (24,856 part files) takes 153 s on
an M-series Mac and 8.0 million boxes, about 96 MB, for parts most users never place; 6A can still prebuild the common
ones into the shipped index if first runs show a need. A common part is small (a 2 x 4 brick is 5 boxes, a 45° slope
82, a straight track 553) and builds in milliseconds; the slowest, a few large printed or curved parts, take under a
second.

### Broad and narrow phase

parry3d-f64 (`default-features = false`, `dim3`, `f64`): a `Bvh` over the parts' world boxes gives candidate pairs.
Each pair's boxes are compared in the first part's frame: by exact interval overlap when the second is turned by
quarter turns (almost every pair), by parry3d's `query::contact` on cuboids otherwise. The finding's depth and axis
come from the parts' body boxes, so the hint says which way to move.

## Measured

- The bench (`just bench validator`, release, warm, M-series): 14 ms for 510 parts and 62 ms for 2,078.
- The nightly corpus (33 models): no error findings; mutation recall 95.9 %, and 98.2 % on the PR gate's corpus.

## Consequences

- A part with a broken mesh (an odd crossing count in every column) has an empty proxy and never collides.
- A new Emmet version that changes `proxy.rs` must bump the index schema so old proxies rebuild.
