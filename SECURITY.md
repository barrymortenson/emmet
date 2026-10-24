# Security policy

## Reporting a vulnerability

Please report security problems privately through GitHub:
[Security → Report a vulnerability](https://github.com/barrymortenson/emmet/security/advisories/new). Don't open a
public issue for them.

Include what you found, how to reproduce it, and what an attacker could do with it. You'll get a reply within a week.
Fixes are released as soon as they're ready, and the advisory credits you unless you'd rather it didn't.

## Supported versions

Emmet is before 1.0. Only the latest release gets security fixes.

## Scope

In scope: the desktop app, the `emmet` CLI and MCP server, the Python DSL, the release and update pipeline, and this
repository's CI. Out of scope: the Claude Code and Codex CLIs themselves (report those to Anthropic or OpenAI), and
problems that need an attacker who already controls the user's account or machine.

## What Emmet does to stay safe

Plan section 15 (`docs/plan.html`) lists the threats and their controls: workspace trust for shared projects, an
isolated Python with timeouts, a strict CSP, path checks on every IPC call, signed updates, gitleaks, and secret
scanning.
