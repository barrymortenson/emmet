# 007: The chat is a sidebar on the left, with the conversation always shown

The frozen design docks the chat along the bottom of the viewer: a short card with the latest line, and a History
button that opens the conversation upward. Barry, before session 5A (2026-11-10): "switch the chat interface panel and
history to the left as a side bar similar to the panel on the right, keep the drawing area open in the middle", and
"not have a history collapse or collapse option, just keep it expanded".

## Changes

| Where | Frozen | The app | Why |
|---|---|---|---|
| The window | The viewer on the left with the chat docked under it; the options panel on the right | The chat sidebar on the left, the viewer in the middle, the options panel on the right | Barry's ask: the model keeps the whole height of the window |
| The chat's width | The viewer's width | 340 wide to start, dragged between 300 and 520 on a separator like the panel's, and kept per user; a narrow window caps it at about a third of its width | "Similar to the panel on the right"; the viewer keeps its place at the smallest window (960 × 600) |
| The conversation | Closed, behind History, which opens upward over 58% of the column, with an edge to drag and Hide history | Always shown, filling the sidebar between its top row and the composer; no History button, no edge, nothing to fold | Barry's ask |
| The latest line | The agent's newest step or words, one line while it works, three lines with Show all and Hide between turns | Gone: the conversation shows it | It stood in for the closed history |
| The chat's top row | The picker, the plan badge, the status with Stop, Undo turn, History, Start fresh | The picker, the plan badge, Start fresh (its icon; the words are its tooltip and name) | What fits a sidebar's width |
| The status and Undo turn | In the chat's top row | In a row of their own under the conversation, over the card that needs you and the composer | The same, where the eye already is while typing |
| The card that needs you | Under the top row | Under the conversation and the status, over the composer | The same order as the frozen history-open screen |
| The agent picker's menu and the plan usage | Open upward | Open downward | They're at the top of the window now |
| The title row | Over the viewer | Over the chat and the viewer: the project chip after the traffic lights, the view switch centered over the viewer. Windows: unchanged, across the window | The traffic lights sit over the chat's column |
| View menu | Show Chat History | Gone, on both platforms | Nothing to show or hide |
| Screen 14, "The history opened" | A screen of its own | Every screen with a conversation is that screen | |

Tokens: `--chat-width` is new; `--dock-history` stays for the frozen screens, which read the app's tokens, and the app
no longer uses it. The files in `docs/design/frozen/` keep the signed design as
it was drawn, as after notes 001 to 006; this note, plan section 14, and `components.md` say what the app does.

## Barry's yes

Asked for (Barry, 2026-11-10). He looked at the app with the change and said the sidebar looks good (Barry,
2026-11-11).
