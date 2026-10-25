# Courtesy note to Rebrickable (draft for H-24)

Barry sends this from his own account (plan section 22, H-24) and records the reply here. Its reply is needed before
3B merges (Q-14); if there's none by then, Barry decides whether 3B merges anyway, in `docs/decisions.md`.

Where to send it: the Contact link in the footer of rebrickable.com, from Barry's own email.

---

Subject: Emmet: a free, open-source app that uses Rebrickable's catalog downloads

Hello,

I'm building Emmet, a free, open-source (AGPL-3.0) desktop app for macOS and Windows. A person describes a brick
model, and their own coding agent (Claude Code or Codex) builds it from real LDraw parts. The source is public at
github.com/barrymortenson/emmet. I'd like to tell you how it uses your data, and ask one question.

How Emmet uses the catalog downloads:

- Once a week, Emmet's CI downloads the catalog archive once and builds a small parts data pack: parts, colors, part
  and color mappings, part relationships, elements, and how many sets each part appears in per color. No MOC data and
  no images; the image URL columns are dropped.
- The pack is published as a release on Emmet's GitHub repository and bundled with the app. Users' machines never
  contact Rebrickable.
- Wherever the data appears (the parts list, exports, the About and Settings screens, and the website), Emmet says
  "Part, color, and set data from Rebrickable (rebrickable.com)".

My question: your Terms say Rebrickable content may not be used to train AI models. Emmet never trains a model. But
while the user's agent builds a model, Emmet's tools answer its questions with small facts from the pack, such as a
part's name or whether it exists in a color, and those answers reach the AI provider the user has chosen. Is that
acceptable to you?

Thank you for the catalog, and for making it available like this.

Barry
