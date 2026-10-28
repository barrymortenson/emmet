# Emmet's keyboard map and menus

Final (plan session 2B). It replaces section 14's draft: no command palette (Barry 2026-10-27), and the keys the
frozen screens show. Every action is in a menu, so everything is reachable from the keyboard; the shortcuts are for
what people do often. 4A's keyboard E2E spec checks every row here. On Windows, `⌘` is `Ctrl` and `⌥` is `Alt`.

## Shortcuts

| Keys (macOS) | Keys (Windows) | Does | Where it works |
|---|---|---|---|
| `⌘N` | `Ctrl+N` | New project | Anywhere |
| `⌘O` | `Ctrl+O` | Open a project or an LDraw file | Anywhere |
| `⌘S` | `Ctrl+S` | Save version | Anywhere |
| `⌘E` | `Ctrl+E` | Export | Anywhere |
| `⌘,` | `Ctrl+,` | Settings | Anywhere |
| `⌘L` | `Ctrl+L` | Go to the message box | Anywhere |
| `↩` | `Enter` | Send | The message box |
| `⇧↩` | `Shift+Enter` | A new line | The message box |
| `⌘↩` | `Ctrl+Enter` | Send | The message box, and while a card has focus |
| `Esc` | `Esc` | Close the open menu, popover, or dialog; with none open, stop the agent (a pending redirect goes out as the next turn) | Anywhere |
| `1`–`5` | `1`–`5` | Pick an answer (the last is Other) | A question card with focus |
| `1`–`3` | `1`–`3` | Allow once, Allow for this session, Don't allow | An approval card with focus |
| `1` `2` `3` `4` `5` `6` | the same | View: Iso, Front, Top, Back, Left, Right | Outside a text field |
| `F` | `F` | Frame the selection, or the whole model | Outside a text field |
| `←` `→` `↑` `↓` | the same | Orbit 15° | The viewer with focus |
| `+` `−` | the same | Zoom in, out | The viewer with focus |
| `[` `]` | `[` `]` | Previous step, next step (opens Steps) | Outside a text field |
| `⌘1`–`⌘4` | `Ctrl+1`–`Ctrl+4` | Options, Checks, Parts, Steps | Anywhere |
| `⌥⌘Z` | `Ctrl+Alt+Z` | Undo the agent's last turn | Anywhere |
| `⌘Z` `⇧⌘Z` | `Ctrl+Z` `Ctrl+Y` | Undo and redo typing, and a knob's draft before Keep | A text field, the panel |
| `F6` | `F6` | Move between the viewer, the chat, and the panel | Anywhere |
| `⌘/` | `Ctrl+/` | Keyboard shortcuts (this list) | Anywhere |
| `Tab` `⇧Tab` | `Tab` `Shift+Tab` | Next and previous control | Anywhere |

Arrow keys move inside a control (tabs, segmented controls, chips, menus, the Steps tree, a slider, a resizable
separator); `Space` and `Enter` press. A key that types a character (`1`, `F`, `[`) never fires while a text field has
focus.

Changes from section 14's draft: `⌘K` is gone (no palette); `⌘E` exports and `⌘,` opens Settings; `Enter` sends, as
chat apps do, with `⇧↩` for a new line and `⌘↩` kept; `Esc` closes what's open before it stops the agent; the view keys
are named (1 Iso, 2 Front, 3 Top, 4 Back, 5 Left, 6 Right); and `⌥⌘Z`, `F6`, and `⌘/` are new.

## Menus

macOS shows these in its menu bar, in title case as macOS does; Windows shows them in the title row's menu bar
(`Alt` focuses it), in sentence case. The project chip's menu repeats File's most-used items.

| macOS | Windows | Holds |
|---|---|---|
| **Emmet Builder** | (Help and File hold these) | About Emmet Builder · Settings… `⌘,` · Check for Updates… · Install Command Line Tool… · Hide, Hide Others, Show All · Quit `⌘Q` |
| **File** | **File** | New `⌘N` · Open… `⌘O` · Open Recent ▸ · Save Version `⌘S` · Versions… · Export… `⌘E` · Close Window `⌘W` · (Windows: Settings… `Ctrl+,` · Exit `Alt+F4`) |
| **Edit** | **Edit** | Undo `⌘Z` · Redo `⇧⌘Z` (`Ctrl+Y`) · Cut · Copy · Paste · Select All · Undo the Agent's Last Turn `⌥⌘Z` |
| **View** | **View** | Iso `1` · Front `2` · Top `3` · Back `4` · Left `5` · Right `6` · Frame `F` · Options `⌘1` · Checks `⌘2` · Parts `⌘3` · Steps `⌘4` · Previous Step `[` · Next Step `]` · Show Chat History · Enter Full Screen `⌃⌘F` (Windows: `F11`) |
| **Agent** | **Agent** | Go to Message Box `⌘L` · Send `⌘↩` · Stop `Esc` · Start Fresh · Switch to Codex (or Claude) · Model and Effort… · Plan Usage… |
| **Window** | (none) | Minimize `⌘M` · Zoom · Bring All to Front |
| **Help** | **Help** | Emmet Guide · Keyboard Shortcuts `⌘/` · Copy Diagnostics · Report a Problem… · (Windows: Check for updates… · Install command line tool… · About Emmet Builder) |

The project chip's menu: New, Open…, Save version, Versions…, Export…, About Emmet Builder.
