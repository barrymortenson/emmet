# Provider terms

The published text behind plan section 10's terms matrix and section 12's data terms (D-37), as short exact phrases
with their URLs. `quotes.json` is the record; `scripts/terms-check` re-reads each page and reports a phrase that's
gone. Run it by hand at G1, G2, and 7B (never in CI). A gone phrase holds the decision it supports until Barry has
settled what changed.

Rebrickable's pages are checked by hand: its Terms (section 5.3) forbid using automation tools to automate traffic to
the site, apart from the catalog downloads. `terms-check` prints them as a checklist.

## 2026-10-24 (0B)

- **Anthropic** ([legal and compliance](https://code.claude.com/docs/en/legal-and-compliance)): every phrase section
  10 quotes is still there. The page also says products may accurately say "in plain text" that they run Claude Code.
  H-20: Barry accepted the Commercial Terms through his Console account on 2026-10-24; the description of how Emmet
  runs Claude Code (`anthropic-description.md`) waits for the website and a business email, and goes out before 7B
  merges.
- **OpenAI** ([overview](https://developers.openai.com/siwc/token-sharing-open-source),
  [sign-in](https://developers.openai.com/siwc/token-sharing-open-source/sign-in),
  [app-server](https://learn.chatgpt.com/docs/app-server)): two of section 10's phrases have changed since planning.
  "is available to all open-source partners" is no longer on the overview, which now describes ChatGPT plan usage
  for open-source and locally hosted apps (with an interest form for paid or remotely hosted ones). "without a client
  secret or partner API key" moved to the sign-in page as "needs neither a client secret nor a partner API key".
  The meaning of the 1E row is the same, and Barry chose to quote today's text (2026-10-24); section 10 now does.
- **Rebrickable** ([downloads](https://rebrickable.com/downloads/), [terms](https://rebrickable.com/terms/)): read in
  a browser (both pages answer scripts with a Cloudflare challenge). Every phrase section 12 quotes is still there.
  The downloads page now also offers the complete catalog as one archive.

## Files

- `quotes.json`: the phrases, their URLs, and what each one supports.
- `rebrickable-note.md`: the courtesy note to Rebrickable for Barry to send (H-24).
- `anthropic-description.md`: the description of how Emmet runs Claude Code, for Barry to send later (H-20).

These are readings of published terms for building Emmet, not legal advice.
