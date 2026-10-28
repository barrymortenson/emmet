# Emmet's icons

The icon set (plan session 2B): [lucide](https://lucide.dev) through `lucide-react` (D-29), plus a few of Emmet's
own brick glyphs in [`apps/desktop/src/assets/glyphs/`](../../apps/desktop/src/assets/glyphs/), drawn on lucide's grid
(24 units, stroke 2, round caps and joins) so the two sit together. Icons are always paired with words or a tooltip,
`aria-hidden`, and drawn in the text color of what holds them. Sizes are tokens: `--icon-sm` 13, `--icon-md` 16,
`--icon-lg` 20, `--icon-xl` 24.

The frozen pages draw them from `docs/design/frozen/generated.js`, which `node scripts/design-frozen.mjs` writes from
`lucide-react` and the glyph files, so the names below are the ones 4A imports (`rotate-ccw-clock` is
`RotateCcwClock`). Use lucide's current names, not its old aliases (`history` is now `rotate-ccw-clock`).

## Emmet's glyphs

| Glyph | Meaning | Used for |
|-------|---------|----------|
| `brick` | A brick: a part | Parts, part counts, first run's "real parts" |
| `plate` | A plate: a thin part | The parts list's kind, where it matters |
| `baseplate` | What a model stands on | The Base build setting |
| `submodel` | A sub-assembly, built apart and used one or more times | The Steps tree's Built apart rows |
| `steps` | Bricks stacked in build order | Steps, the instructions export |
| `floating` | A part held by nothing below it | The floating finding and its pin |
| `collision` | Two parts in the same space | The collision finding |

## lucide

| Meaning | Icon |
|---------|------|
| Send | `send` |
| Stop | `square` |
| Open, more (a menu or a choice) | `chevron-down` |
| Done, picked | `check` |
| Clean, no problems, saved | `circle-check` |
| A problem to look at | `triangle-alert` |
| Broken: a failed build, a crash | `octagon-alert` |
| Information, the empty state | `info` |
| Close, dismiss, a failed tool | `x` |
| Building (a run) | `box` |
| Searching parts | `search` |
| Reading the rules or the project | `book-open` |
| Looking at the model (a render) | `eye` |
| A change to the model, in words | `file-pen` |
| Build settings | `sliders-horizontal` |
| A list of parts (export) | `list`, `sheet` (CSV) |
| An instructions file | `file-text` |
| Download, export, update | `download` |
| An imported file | `file-up` |
| Open a project | `folder-open` |
| Opens a page in the browser (Manage usage) | `external-link` |
| New | `plus` |
| Save a version | `save` |
| Versions, a version open | `rotate-ccw-clock` |
| Undo a turn, restore | `undo-2` |
| Try again, rebuild, restart | `refresh-cw` |
| Shuffle a knob's arrangement | `shuffle` |
| Waiting (a pending redirect) | `clock` |
| History | `messages-square` |
| Start fresh | `message-square-plus` |
| A question waiting | `message-circle-question-mark` |
| An approval waiting | `shield-question-mark` |
| The plan's limit | `gauge` |
| Sign in | `log-in` |
| Read-only, trust | `lock` |
| Offline | `wifi-off` |
| Attach a photo | `paperclip` |
| Frame the model | `scan` |
| Screenshot | `camera` |
| Copy | `copy` |
| The command line tool | `terminal` |
| Windows' caption buttons | `minus`, `square`, `x` |

## The app icon

The final icon is [`icon/app-icon-macos.svg`](icon/app-icon-macos.svg) (the ant on a warm plate, on Apple's icon
grid) and [`icon/app-icon-windows.svg`](icon/app-icon-windows.svg) (the ant alone, with heavier legs for 16 px).
`node scripts/app-icon.mjs` makes every platform size with `tauri icon` into `apps/desktop/src-tauri/icons/`:
`icon.icns` (16 to 1024 px, with the @2x pairs), `icon.ico` (16, 24, 32, 48, 64, 256), and `32x32.png`,
`128x128.png`, `128x128@2x.png`, and `icon.png` (512). The brand's other marks stay in [`brand/`](brand/).
