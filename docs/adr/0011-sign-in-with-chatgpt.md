# ADR 0011: Sign in with ChatGPT as built

Status: accepted, 2026-10-26 (session 1E)

## Context

Session 1E runs Codex in the chat pane on the user's ChatGPT plan through OpenAI's Sign in with ChatGPT flow for
open-source apps (D-35, plan section 10). OpenAI's pages (docs/terms/, read 2026-10-26) specify the flow, the
app-server provider, the errors, and the UI. This ADR records how Emmet meets them, and the few places it chose
between options the docs leave open. `docs/spec/sessions/1E.md` maps each requirement to its test.

## Decisions

### Dependency (approved by Barry, docs/decisions.md 2026-10-26)

| Crate | Version | License | For |
|---|---|---|---|
| ring (default features off, `alloc`) | 0.17.14 | Apache-2.0 AND ISC | RS256 ID-token signatures, SHA-256 for PKCE, and the system random source for state, nonce, verifier, and the host id. Already in the tree through ureq and rustls; the fakes use it to sign test ID tokens |

ureq (ADR 0007) makes the HTTPS calls. The loopback listener and query parsing are `std`. Rejected: RustCrypto's `rsa`
with `rand` (about 10 new crates) and `jsonwebtoken`.

### Where things live

- `emmet-agent::chatgpt`: the host id (`chatgpt/host.json` in app data, `urn:uuid:` + a UUIDv4, OpenAI's supported
  alternative to a key thumbprint, which would need key storage for no gain since OpenAI doesn't verify it), the
  registrations (`chatgpt/accounts.json`: client id, `sub`, email, label, plan on or off, signed in, notice seen), the
  flow, refresh, and revocation.
- Credentials: one keychain entry per issued client id (`Emmet` / `chatgpt:<client id>`), holding OpenAI's example
  record as JSON. OpenAI's sign-in page shows a 0600 file; its accounts page asks for "protected local or self-hosted
  runtime storage". The keychain is that, keeps tokens out of every file, and matches how 1C keeps the API key.
  Detection and the picker read only `accounts.json`, never a secret (reading one prompts for the login password on
  macOS whenever a dev build's signature changed).

### The flow

- Port 1455 first (OpenAI's example), any free loopback port after it. The listener answers a request with another
  state and keeps waiting: a stale tab or a forged callback can't end or hijack the attempt. The wait is 10 minutes,
  and Cancel stops it.
- The browser opens through `open` (macOS) or `rundll32 url.dll,FileProtocolHandler` (Windows), spawned in Rust. The
  window can't open URLs; Manage usage is a typed command that opens `https://chatgpt.com/settings/usage`.
- ID tokens: RS256 only (the header's `alg` is not trusted), the key by `kid` from the JWKS that discovery names, the
  issuer, the audience (the issued client id), `exp` with 60 s of skew, and the nonce.
- A first registration whose code exchange fails with `invalid_grant` keeps its issued client id as a pending
  registration (hidden from the picker), so the retry reauthorizes it instead of registering again.
- Enabling plan use after a decline repeats OAuth with `prompt=consent`. OpenAI's `force_reconsent=true` waits on
  OpenAI confirming deployment for the integration.
- Labels: the email, made unique ("… (2)") when two registrations share it (two workspaces), and kept after that.

### Renewal

- Refreshes hold an in-process lock and a file lock (`chatgpt/refresh.lock`, std's `File::lock`), then reread the
  record, so two windows or processes never spend one rotating refresh token twice.
- A Codex session renews only between turns: before a turn that would start with less than 30 minutes of token left,
  or after the API refused the token (a 401). It then restarts the app-server with the new `ACCESS_TOKEN`,
  initializes, and `thread/resume`s the saved thread (OpenAI's rule). The chat never sees the old process's exit. A
  token never expires mid-turn unless a turn runs past 30 minutes; then the turn fails with the sign-in error and the
  next turn renews. Emmet never starts a turn on its own (plan section 10).
- `earliest_refresh_at` is honored. A network failure or 5xx keeps the credentials and uses a token that's still good.
  An unusable refresh token clears the access and refresh tokens (the ID token stays for `id_token_hint`), and the
  account shows as signed out.

### The app-server and errors

- OpenAI's provider block, word for word (`openai_chatgpt_plan`, including `supports_websockets=false`, which Codex
  0.160.1 accepts with `--strict-config`). `clientInfo.name` is "Emmet" for both providers, matching `agent_name_hint`.
- OpenAI's documented error codes map to their recovery (`codex.rs` `PLAN_ERRORS`), with Codex's own text (status,
  code, request id) kept after it. A usage limit is the limit card; nothing switches to the API key.
- Codex's choice is explicit: signing in chooses the ChatGPT account, and Settings can choose the API key instead. A
  ChatGPT choice that can't be used (signed out, plan use off) is an error with its fix, never a fallback.

### Secrets in outputs

The engine takes a session's secrets (the API key, or the access token) and replaces them with `<redacted>` in every
transcript line and every event, wherever the CLI echoed them. A test seeds tokens through fake-openai, has the fake
CLI echo them on stderr, in an error, and in text, and greps every file Emmet wrote and every event; the E2E spec does
the same over the app's data, the project, and the run's output. Emmet has no log file and no diagnostics bundle yet;
6B's bundle must join both tests.

### The UI (plain until 4B)

OpenAI's words throughout. "Continue with ChatGPT" is OpenAI's approved continue-black format (black, 45 px, 12 px
radius, 242 px, OpenAI's white ChatGPT logo shipped unchanged), in both themes: the white format would need a black
logo. The notice shows once per account. The limit card has no second action: Emmet sells no credits. Pricing-page
eligibility and self-hosted VMs don't apply: Emmet has no plans or prices and runs only on the user's machine.

## Consequences

- A `fake-openai` in `tests/fake-agents` (`EMMET_FAKE_OPENAI`) and a fake browser (`EMMET_FAKE_BROWSER`), debug builds
  only, like `EMMET_FAKE_KEYCHAIN`.
- The window gains seven typed commands (`chatgpt_*`, `open_chatgpt_usage`); none takes a URL. The CSP is unchanged.
- 4B restyles the notice, the label, the limit card, and the picker; the button keeps OpenAI's format.
