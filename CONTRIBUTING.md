# Contributing to Emmet

Thanks for helping. Emmet is built in planned sessions (`docs/plan.html`, section 21), so for anything larger than a
fix, open an issue first and say what you'd like to change.

## Setup and checks

```sh
just setup   # checks the pinned toolchain, installs the git hooks, installs dependencies into the repo
just check   # format, lint, typecheck (seconds)
just test    # integration tests
just ci      # everything CI's mac job runs; must pass before you push
```

The README lists the tools `just setup` expects.

## Rules for changes

- Integration tests first: prove behavior through the real `emmet` binary, the real app, or the real Python. Every
  bug fix comes with a test that would have caught it. Never delete, skip, or weaken a test to get green.
- Every new source file starts with an SPDX header: `SPDX-License-Identifier: AGPL-3.0-or-later`, or
  `SPDX-License-Identifier: Apache-2.0` under `python/emmet_dsl/`, `data/techniques/`, and the project templates.
  CI checks it (`node scripts/spdx-check.mjs`).
- No new dependency, tool, or service without discussing it in an issue first.
- No secrets, tokens, real names, emails, or personal paths in code, fixtures, transcripts, or screenshots.
- US English (color, gray, behavior).

## License of contributions

Your contribution is licensed under the license of the files it touches: AGPL-3.0-or-later for most of the repo, and
Apache-2.0 for `python/emmet_dsl/`, `data/techniques/`, and the project templates. There is no CLA.

## Sign off every commit (DCO)

By signing off, you certify the Developer Certificate of Origin below for that commit. Add the sign-off with `-s`:

```sh
git commit -s -m "Fix the stud count in the BOM"
```

That appends `Signed-off-by: Your Name <you@example.com>` using your git `user.name` and `user.email`, and it must match
the commit's author. To keep your email private, use your GitHub noreply address
(`git config user.email ID+USERNAME@users.noreply.github.com`). To fix commits that are missing it:

```sh
git commit --amend --no-edit -s     # the last commit
git rebase --signoff origin/main    # every commit on your branch
```

CI's DCO check (`scripts/dco-check.sh`) fails a PR when any commit lacks a matching sign-off.

## Pull requests from forks

CI runs on GitHub-hosted runners for forks, after a maintainer approves the run. The PR template asks for a Done when
list with evidence; fill in what applies.

## Developer Certificate of Origin

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.

```
