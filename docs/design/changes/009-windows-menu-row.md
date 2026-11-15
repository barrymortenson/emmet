# 009: On Windows the menus get a row of their own, over a title row like macOS's

The frozen design puts Windows' menu bar in the title row, between the project chip and the view switch, with the
caption buttons at its end. On Windows 11 (Barry, 2026-11-15) the menus read as loose text dropped between the
floating pills: "the top file bar menu needs to be designed so it looks more like a modern windows app not just
dropped in". Offered a flat WinUI row, two rows, or the menus in a pill, he chose: "two rows with the file menu on the
top and the other options just like they look on the mac".

## Changes

| Where | Frozen | The app | Why |
|---|---|---|---|
| The window's top, Windows | One 48 px title row across the window: the chip, the menu bar, the view switch, the caption buttons | Two rows: a 32 px menu row across the window (File, Edit, View, Agent, Help on the left, the caption buttons on the right, flat on the canvas), then the 56 px title row as on macOS: the project chip at the left, the view switch centered over the viewer | Barry's choice |
| The title row, Windows | Across the window, over the options panel | Over the chat and the viewer, as on macOS; the options panel starts under the menu row | "Just like they look on the mac" |
| The caption buttons | The title row's height | The menu row's height (32), as Windows 11 apps draw them | |
| The title row, both platforms | Its labels could be selected (Ctrl+A or a drag highlighted File, Edit, the views, and the project's name) | Never selected: it's the window's frame | The highlight in Barry's screenshot |

Tokens: `--menubar-height` (32) is new, Windows only; Windows' `--titlebar-height` is 56, as macOS's. The frozen files
keep the signed design as drawn; this note, plan section 14, and `components.md` say what the app does.

## Barry's yes

Asked for, and the layout chosen (Barry, 2026-11-15). To confirm on the Windows VM.
