# 4C: does an agent notice a missing roof tile in its render?

What good looks like 1: "Renders handed to the agent are clear enough that it notices a missing roof tile."

## How

- The Cottage (`emmet new`, 905 parts) rendered through `viewer.render` at 768 × 768 from Iso, as `emmet render` and
  the MCP `render` tool hand it to an agent: `roof-tile/whole.png`, and the same with one tile of the front roof
  slope taken out (`roof/roof-course-3-front/3`, a 2 x 4 slope behind the dormer): `roof-tile/missing.png`. The two
  differ in 328 pixels, at (335–383, 316–339).
- A fresh Claude session (Claude Code's general-purpose agent, 2026-11-01, no access to this session's context) got
  only `missing.png`, copied to a neutral name (`cottage-iso.png`), and this prompt, with no hint of what changed:

  > You are reviewing a render of a brick model (built from real LEGO-compatible LDraw parts) that a coding agent just
  > built. […] Task: list every building problem you can see in this model — anything missing, misplaced, floating,
  > overlapping, or that a careful builder would fix. Be specific about where each problem is […]. Answer in a short
  > list, most important first.

  No replay or eval was run for this (D-38: no scripted evals mid-session).

## What it said (first items, verbatim)

> 1. **Hole in the main roof behind the dormer** (upper center, just above the small dormer). A white rectangular
>    gap with a brown plate edge shows through the red roof slopes. The dormer doesn't join the main roof.
> 2. **Ragged right edge of the main roof, next to the chimney** (center right). […]
> 3. **Messy picnic area in the front yard** (lower center). […]

Its first and most important finding is the missing tile, placed correctly. The rest (the roof's stepped edge at
the chimney, the yard's arrangement, studs left showing on the small roofs, the tree's shape) are about the Cottage
as designed, and are in both renders.

Result: passes. The renders show a single missing 2 x 4 slope clearly enough that an agent finds it first, unprompted.
