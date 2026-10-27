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
  H-20: Barry accepted the Commercial Terms through his Console account on 2026-10-24.
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

## 2026-10-25: best reading, not written confirmation

Barry amended D-37: providers rarely answer questions about their terms, so where the published text doesn't settle
a use, Emmet builds on Barry's best reading of it, records the reasoning here, and shows users a notice that use is at
their own risk. No description goes to Anthropic (H-20 is done with the Commercial Terms accepted), and no question
goes to Google.

- **Google** ([Gemini CLI terms](https://github.com/google-gemini/gemini-cli/blob/main/docs/resources/tos-privacy.md),
  [FAQ](https://geminicli.com/docs/resources/faq/),
  [quotas](https://geminicli.com/docs/resources/quota-and-pricing/),
  [Antigravity headless mode](https://antigravity.google/docs/cli/headless/)): Gemini CLI "was replaced by
  Antigravity CLI on June 18th, 2026" for unpaid and Google One users. Google forbids software that will "harvest or
  piggyback on Gemini CLI's OAuth authentication to access our backend services". Antigravity's headless mode is
  documented to "script agent tasks, integrate with CI pipelines, and capture machine-readable output", and "uses
  your cached credentials". Barry's reading (D-42): the prohibition is about software that takes the CLI's sign-in to
  call Google itself. Emmet starting the user's own unmodified `agy` in its documented headless mode, on the sign-in
  the user gave `agy`, for turns the user starts, is use of Google's CLI, not of its sign-in; Google did the same with
  Zed in 2025 ([Zed's post](https://zed.dev/blog/bring-your-own-agent-to-zed)). Terminal mode (1F) and the chat pane
  (4F) are built on that. Two Google forum replies say the pattern is supported, from accounts with no Google role
  shown, so they aren't relied on. 4F adds these phrases to `quotes.json`.

## 2026-10-26 (1E)

- **OpenAI** (Sign in with ChatGPT, all ten pages under
  [token-sharing-open-source](https://developers.openai.com/siwc/token-sharing-open-source) and the
  [UI/UX guidelines](https://developers.openai.com/siwc/ui-ux-guidelines)): every phrase recorded in 0B is still
  there, and the overview still says the docs explain "ChatGPT plan usage for open-source and locally hosted apps".
  The repo is public under AGPL-3.0, so 1E goes ahead. New phrases for what 1E builds on (the first-use notice, the
  composer label, the limit card, the account picker, serialized refresh, revocation, the app-server provider, and
  the error rules) are in `quotes.json`.
- One difference from the plan's wording: the sign-in page's example keeps the credential record in a local file
  (0600) under an app-defined path. Its accounts page asks for "protected local or self-hosted runtime storage".
  Emmet keeps the record in the OS keychain, as section 10 says, which is at least as protected and keeps the tokens
  out of every file (ADR 0011).
- The approved "Continue with ChatGPT" button is black (or white) with OpenAI's ChatGPT logo. Emmet ships that logo
  unchanged, only inside the button (Barry 2026-10-26; TRADEMARKS.md).

## 2026-10-26 (1F)

Installing each CLI for the user and running Claude Code's sign-in from Emmet keep their basis; every earlier phrase is
still there.

- **Anthropic** ([legal and compliance](https://code.claude.com/docs/en/legal-and-compliance)): "preinstalling or
  running Claude Code in your products" under the Commercial Terms (H-20), with the binary "installed and run as
  published by Anthropic", and sign-in that "must complete through Anthropic's own flow". The
  [setup page](https://code.claude.com/docs/en/setup) publishes the native installer Emmet runs word for word, says
  "You do not need to run as Administrator", lists SHA-256 checksums per release (which the installer checks), and
  says the macOS build is "signed by \"Anthropic PBC\" and notarized by Apple" and the Windows one by "Anthropic, PBC",
  which Emmet checks after an install. The [authentication page](https://code.claude.com/docs/en/authentication)
  documents `claude auth login` "in your shell before your first session": Emmet runs exactly that, with nothing on
  its stdin. When the browser can't reach Claude Code's callback, the page shows a code to paste at "Paste code here
  if prompted"; Emmet never takes that code and offers the Terminal instead (Barry 2026-10-26).
- **OpenAI**: Codex "is licensed under the Apache-2.0 License" ([repo](https://github.com/openai/codex)), which
  publishes the standalone installers Emmet runs word for word, with their documented `CODEX_NON_INTERACTIVE`. Each
  checks its archives' SHA-256. OpenAI publishes no signing identity for Codex, so Emmet relies on the installer's
  checksums. [Pricing](https://learn.chatgpt.com/docs/pricing): Codex is "included in your ChatGPT Free, Go, Plus,
  Pro, Business, Edu, or Enterprise plan".
- **Google** ([MCP](https://antigravity.google/docs/mcp/)): Antigravity adds MCP servers through its "Interactive MCP
  Manager" or two files, the global `~/.gemini/config/mcp_config.json` and a workspace's `.agents/mcp_config.json`.
  Emmet writes only the project's file, on a click (Barry 2026-10-26), so terminal mode stays ordinary use (D-42).

## Files

- `quotes.json`: the phrases, their URLs, and what each one supports.

These are readings of published terms for building Emmet, not legal advice.
