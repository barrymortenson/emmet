# 005: What session 4D's photos add to the frozen screens

The frozen composer draws "Attach a photo" (`paperclip`) and nothing after it: what an attached photo looks like, the
first-use sentence, a sent photo, and the compare view aren't in the frozen screens. The app does the following, in the
frozen design's own components and tokens. The screenshots in `docs/spec/sessions/4D/` show each state in light and
dark.

## Changes

| Where | Frozen | The app | Why |
|---|---|---|---|
| Attach a photo | The icon button, first in the composer's bar | The same; it opens the file picker (JPEG, PNG, HEIC), and a photo can also be pasted into the message box or dropped on the composer, which takes the accent edge while a photo is over it | 4D's "drag, paste, or pick" |
| An attached photo | Not drawn | A chip above the message box: the photo small, its file name, and Remove (`x`); a spinner chip while it's being made ready | The user sees what will go before Send |
| The first photo | Not drawn | One quiet line above the chips, "Your photo goes to Claude Code, under Anthropic's terms." (or Codex, OpenAI's), with Got it; once per user | 4D's first-use sentence; plan section 15's privacy line |
| Send with a photo | Disabled while the message box is empty | Enabled with a photo and no words; the photo goes as "Build this." The placeholder says "Build this, or say what to make of it" | A photo is a request on its own |
| While the agent works, or a question is open | The composer stays open for a redirect or an answer | The same; Attach a photo waits, and its tooltip says why ("Attach the photo when Claude has finished.") | A photo starts a turn: a redirect and an answer are words |
| A sent photo | Not drawn | The photo small, above the message's words, as a button ("Compare with the model") | Where the compare view opens from |
| The compare view | Not drawn | A card over the viewer's left half: "Drag the model to line it up with your photo", Close (`Esc`), and the photo, fit; the model takes the right half, its camera where the user left it. Pins hide while it's open | 4D: "the photo beside a render that the user lines up by dragging. The camera still never moves on its own." |

## Barry's yes

Yes, as built (Barry, 2026-11-04, through AskUserQuestion, after the screenshots and his hand checks with Claude Code
and Codex).
