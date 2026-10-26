# meta-development

Tracks, budgets and enforces development work for projects that adopt it, starting with itself.
Principles, ladder and design: `context/SPEC.md`. Build stages: `context/BOOTSTRAP.md`.

## Layout

- `crates/`: `meta-schema` (types), `meta-log` (the `meta/log` branch), `meta-state` (folds),
  `meta-listing`, `meta-gh`, `meta-cli` (the `meta` binary and its Claude Code hooks).
- `tooling/`: one crate per lint or task; `tooling/policy.toml` holds every limit and budget.
- `xtask/`: `cargo xtask ci` (fmt, clippy, lints, tests, test-time budget); CI runs it with `--affected`.
- `plugin/`: Claude Code hooks that call `meta hook`. `self/`: lint baseline and log seed scripts.
- `examples/consumer/`: a small workspace that uses the tooling with its own policy; CI runs it.
- `context/` and this file: agent context (prose category 1). `README.md` and `docs/`: STE100
  documentation only (category 2).

## Upgrading

- Rebuild `meta` after pulling main. The first write by a new build adds a `SchemaVersion` event to
  `meta/log`; an older build then stops writing and says to upgrade, so it never folds claims differently.

## Privacy

- Never put personal details (names, emails, trips, personal notes) or local machine details
  (usernames, home paths, hostnames) in files, commits, PR text, or log events.
- Personal or project-specific material lives in that project's own private repo, not here.
- The log actor is `META_ACTOR`, else `owner`. Never derive it from the user or machine.
- Web requests carry no personal data; use a fixed project User-Agent; never send email.

## Process

- Work from the log: `meta list`, `meta claim <task>`, `meta done <task>` once GitHub CI passes.
  Report surprises with `meta issue`; vague worries with `meta concern`. Every new concern, issue
  and task takes `--division <path>` (a path under a top division, see `meta` usage).
- One task, one small PR off `main`. Stack only a truly atomic sequence, and then only with
  `gh stack`, never `--base <branch>`. A small good PR never waits on a large one.
- Push your own branch right after every commit so CI runs.
- Never merge, approve, or push to `main`; the owner merges. Hooks deny merges, force pushes,
  and pushes to protected branches. Only `meta` writes `meta/log`.
- Merge conflicts are expected; merge `main` in and fix them. No hand rebase or force push;
  `gh stack` may restack.
- When a PR tightens a check, the first pushed commit only tightens it, so CI goes red on the
  target; link the red run in the PR, then make it green.
- `cargo xtask ci` passes locally before a PR is done. Report wall time per step; flag any step over 10 min
  or any growth in a gate. Keep headroom: aim for 60% of a budget, not just under it.
- Review agents get the goal and constraints only, never candidate findings.

## Code

- No comments, including `///`. Meaning lives in names and types.
- Make invalid state unrepresentable: types first, then a lint, then a test that lists every failure.
- Newtypes over primitives in signatures and fields; no `serde_json::Value` fields.
- One struct, enum, or trait per file, named after it.
- Policy values (limits, budgets, word lists) live in `tooling/policy.toml`, per project (principle 5).
- Complexity is budgeted: a new type, rule, or hook names the concern it moves up the ladder.
  Prefer deleting code.
- Strings are terse (category 3): errors, help, commit messages, PR titles and bodies.

## Threat model

- Local checks and hooks guard against honest mistakes, not deliberate bypass.
- Make bypasses visible, not impossible: changes to enforcement surfaces (`surfaces` in
  `tooling/policy.toml`) are listed in the CI summary. Server rules (required `ci`, protected
  branches) are the backstop.
