# Description of how Emmet runs Claude Code (draft for H-20)

Barry sends this to Anthropic through the "contact sales" link on the Claude Code legal and compliance page
(https://code.claude.com/docs/en/legal-and-compliance, under "Authentication and credential use") once Emmet has its
website and a business email, and before 7B merges (docs/decisions.md, 2026-10-24). Before sending, check it still
matches plan section 10, and drop "free" if Emmet might ever charge. The reply goes in this folder.

---

Subject: Emmet: running the user's own Claude Code inside an open-source desktop app

Hello,

I'm building Emmet, a free, open-source (AGPL-3.0) desktop app for macOS and Windows that runs on the user's own computer. The user describes a brick model, and their own coding agent builds it by writing a small Python model from real LDraw parts, with a live 3D view. The source is public at github.com/barrymortenson/emmet.

I've accepted the Commercial Terms through my Console account. I'd like you to confirm that the way Emmet runs Claude Code, described below, is within your terms, per the "Can customers offer Claude Code in their products?" section of the Claude Code legal and compliance page.

How Emmet runs Claude Code:

- It uses the Claude Code the user already installed, unmodified. Emmet doesn't bundle, patch, or redistribute the binary.
- It starts claude -p with stream-JSON input and output in the user's project folder, with an MCP config that adds Emmet's own tools (search parts, run and check the model, render, ask the user a question), --permission-prompt-tool for approvals, and --allowedTools and --disallowedTools to scope edits to the model files.
- Sign-in is left entirely to Claude Code. Emmet never passes --bare, never sets or filters credential variables, never reads token files or the keychain, and runs with the same environment as the user's terminal. A user who isn't signed in is told to run claude and /login themselves.
- Each user is billed on their own account: their own Pro or Max plan, or an API key they already have in their environment. Emmet doesn't pay for, resell, or proxy any Claude usage, and it shows which one is in use ("Claude · Max plan" or "Claude · API key") from system/init.
- Every model turn starts from something the user did (a message, an answer, a request to fix a finding). Emmet starts no turns on its own.
- Plan usage display: only when the user opens it or a turn they started ends, at most once a minute, from the documented SDKUsageReport (sending /usage) and rate_limit_event. No private endpoints and no background polling.
- Names: "Claude" and "Claude Code" appear in plain text only, as in "runs Claude Code", never in an Emmet feature name, and no Anthropic logos.
- My own automated testing (scripted protocol recordings and evals) runs on my own Console API key, not on a subscription.

Is this within your terms? If any part isn't, I'll remove it before it ships.

Thank you,
Barry
