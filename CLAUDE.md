# meta-development

Tracks, budgets and enforces development work for projects that adopt it, starting with itself.
Principles, ladder and design: `context/SPEC.md`. Build stages: `context/BOOTSTRAP.md`.

## Layout

- `crates/`: `meta-schema` (types), `meta-log` (the `meta/log` branch), `meta-state` (folds),
  `meta-listing`, `meta-gh`, `meta-cli` (the `meta` binary and its Claude Code hooks).
- `tooling/`: one crate per lint or task; `tooling/policy.toml` holds every limit and budget.
- `xtask/`: `cargo xtask ci` (fmt, clippy, lints, tests, test-time budget, supply chain); CI runs it with `--affected`.
  Supply chain: cargo-deny with a config rendered from `[supply_chain]` in `tooling/policy.toml` (into
  `target/meta/deny.toml`; no file to copy), and cargo-semver-checks against the merge base for published crates
  plus `semver`. A breaking change to `meta-xtask` needs an explicit version bump. An advisory fetch failure
  blocks in CI only.
- `plugin/`: Claude Code hooks that call `meta hook`. `self/`: lint baseline and log seed scripts.
- `self/spec` (`meta-spec`): spec rules as typed records; `context/SPEC.md` is rendered from them
  (`cargo run -p meta-spec`). `cargo xtask ci` checks the render in its `spec` step.
- `examples/consumer/`: a small workspace that uses the tooling with its own policy; CI runs it.
- `goldens/`: real recorded git, gh and meta output (`<scenario>.toml` steps, `<scenario>/` recordings).
  `cargo xtask goldens` re-records git and meta ones (CI job `goldens` fails on any diff); gh ones refresh
  only in the scheduled `goldens-online` workflow. Recorded with git 2.54-2.55. Tests feed views, not fakes.
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
- Stacked branches are added with `gh stack add task-<id>-N`, so CI finds the task.
- Push your own branch right after every commit so CI runs.
- Never merge, approve, or push to `main`. The owner or the lead lands PRs with `meta queue run`. Hooks deny merges, `meta queue run` unless `META_ACTOR` names an actor in `tooling/policy.toml` `[queue] landers` (`owner`, `claude-lead`), force pushes,
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
