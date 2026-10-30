# Test architecture: no spawns outside a small scenario set

Status: design for owner review. Task: task-914f9a9748b07f03. Base: main `fd32d8c0`.

## Problem

- The test suite starts many processes: git, the `meta` binary, `gh-replay`, `sh`, `sleep` and `kill`.
- Raw test CPU on CI follows the runner, not the code. Over 30 main runs (368 to 426 tests), CPU was 2.61 s to 4.69 s.
  The mean is 3.27 s and the standard deviation 0.52 s. The blocking line is 4.0 s.
- CPU follows the runner's `git --version` reference (`runner reference`, printed by `test_time`):
  the correlation is 0.90 and CPU ≈ 3.85 × reference. The reference has two levels, about 0.80-0.92 s and about 1.08-1.16 s.
  A PR that lands on the slow level adds about 0.9 s.

## 1. Measured inventory

Method:
- I ran the full suite locally (macOS, git 2.54) through the xtask per-test runner, 3 times, for CPU per test.
- I ran it once more through a counting runner. Each test got its own `GIT_TRACE2_EVENT` directory, which counts every git
  process, nested ones included. A `PATH` shim logged `gh sh bash sleep kill lsof ps cargo true false`. A local-only
  edit (reverted, never committed) logged each start of `meta` and of the replayer.
- Not seen: `/bin/sh` started by absolute path, and helpers that git starts itself. CPU of detached watchers is also
  not seen, because `RUSAGE_CHILDREN` counts only waited children. So the `it` ci figures are low.
- 3 tests fail on macOS until #222 (gc, 2 process_table). Their counts are from the counting run. Their CPU is from CI.

Totals, 427 tests, local:

| item | value |
|---|---|
| test CPU, mean of 3 runs | 5.52 s (5.31-5.76 s); CI main `fd32d8c0`: 2.88 s, ratio 0.52 |
| tests that start at least one process | 55 (13 %) |
| CPU of those 55 tests | 4.20 s (76 %) |
| CPU of the other 372 tests | 1.32 s (24 %); median 2.4 ms, floor 1.1 ms per test |
| processes started by tests | 752: git 521 (274 started directly), kill 47, sh 38, meta 58, sleep 20, bash 18, replay 17, true 17, false 10, ps 5, lsof 1 |
| processes started by the runner | 427, one per test (part of the 1.1 ms floor) |
| CI `git-count`, main `fd32d8c0` | 593 git (304 not under `meta`); budget 652 |

By area:

| area | tests | processes | git | meta | replay | other | local CPU | becomes |
|---|---|---|---|---|---|---|---|---|
| `it` ci (watcher, ci-wait, stop) | 6 | 136 | 95 | 29 | 8 | 4 | 846 ms | in process, 2 scenarios |
| meta-log `git_log` (CAS, races) | 8 | 137 | 137 | 0 | 0 | 0 | 845 ms | in process |
| xtask ci (gate, standing log, stranding, checkouts) | 8 | 113 | 110 | 0 | 0 | 3 | 697 ms | in process |
| `it` cli (writes, claim) | 3 | 70 | 62 | 8 | 0 | 0 | 531 ms | in process, 1 scenario |
| `it` hooks | 3 | 54 | 20 | 15 | 0 | 19 | 284 ms | in process, 2 scenarios |
| meta-golden self-tests | 3 | 33 | 24 | 0 | 9 | 0 | 172 ms | replayer in process; recorder test to the goldens job |
| `it` status | 1 | 28 | 23 | 5 | 0 | 0 | 165 ms | in process |
| xtask revision | 2 | 20 | 20 | 0 | 0 | 0 | 136 ms | in process |
| `it` gc | 1 | 21 | 19 | 1 | 0 | 1 | 226 ms CI | in process, 1 scenario |
| meta-cli project, mutation scope, shard plan | 3 | 12 | 11 | 0 | 0 | 1 | 84 ms | in process |
| port contracts (list below) | 17 | 128 | 0 | 0 | 0 | 128 | 417 ms | stay; allowance per test |

Slowest by processes: `ci_wait_returns_at_the_first_failure…` 48, `a_passing_push_lets_the_agent_stop…` 44,
`post_tool_use_pushes_unprotected_branches…` 40, `standing_log::the_isolated_run_reads_exactly_the_fetched_tip…` 36,
`a_run_ends_at_its_deadline…` 35, `a_claim_failing_after_its_worktree…` 33.

Port contracts are tests whose subject is the process itself: `meta-run` (`run.rs`, placement), `meta-process-table`,
`rusage`, `task::command_line`, `test_time` runner, `mutation` test_packages, monitor `live_machine`, heartbeat
`supervised` and `gc`, xtask `gate_child`/`gate_lock`, and `adoption`.

Comparable in-process tests already exist. They use git2 repos in temp dirs and cost 2-27 ms local:
`claim_run` 9 ms (the `it` claim test is 211 ms), `meta-git` worktrees 20 ms, `collection` 4 ms.

## 2. Target architecture

Rule: a test outside the scenario set and the port contracts starts no process.

1. **In-process entry.** `meta_cli::Call` holds what `main` reads today: args, directory, an `Environment`, stdin.
   `Call::answered()` gives `Answer { out, err, code, then }`. `then` holds follow-up calls as data: for example,
   "watch CI for commit X". `main` prints the answer and starts each `then` as a detached `meta` (the code that today
   calls `current_exe`). A test runs the same `Call` and runs each `then` in process.
   The 14 ambient reads in meta-cli (`current_dir`, `env::var`, `stdin`) move into `Call`.
   The `crate_graph` Environment effect then has one site per binary, and `thin_main` holds.
2. **Git through meta-git only.**
   - The 19 `GitArgv` CLI calls (reflog walk, rev-list, merge-base, status, ls-files, worktree list, `git var`,
     `--git-path`) move to git2 queries in `meta-git` (task-9d6dba62, task-6323c77f).
   - xtask `Revision`, local-gate checkouts and mutation scope use `meta-git` too.
   - `meta_git::Remote` picks its transport from the URL. A plain path uses libgit2 local transport in process, with
     compare-and-swap through `reference_matching`. A URL with a scheme (`https`, `ssh`, `file://`) uses the git CLI,
     as today.
   - Tests build repos with `meta-git` writes (`FileCommit`, `BranchCreate`, `WorktreeAdd`) on temp dirs. `it/fixture.rs`
     shrinks to calls of that API.
3. **gh from recorded replies, in process.** The gh port takes a source: `Spawned(program)` or
   `Recorded(dir)`. `Recorded` reads `goldens/gh-*` through the replay code, moved from `meta-golden` into `meta-run`.
   The `gh-replay` binary stays only for scenarios. The fake `gh` shell scripts in `it/gc.rs` and `it/status.rs` go.
4. **Thin shell.** Decisions take views and return verdicts (#200 does this for the watcher). Tests target the
   verdicts. Each command keeps one in-process `Call` test for its wiring.
5. **What stays real.**
   - Scenario set (target `meta-cli/tests/scenarios`, `test = false`, own CI job). It runs the real `meta` binary on
     `file://` remotes, so the CLI transport stays tested:
     1. post-tool-use push with a slow receive-pack, interrupted: no git helper left alive;
     2. session-start with a hanging upload-pack: answers before its deadline;
     3. post-tool-use starts a detached watcher; concurrent `ci-watch` gives one watcher per commit;
     4. stop with a slow remote returns inside `stop_millis`;
     5. gc keeps a worktree that a live process holds (`lsof`, process table);
     6. `claim --worktree` and a log write through `git push` with lease.
   - The goldens job keeps recording real git, gh and meta output. Its recorder self-test moves there.
   - Port contracts stay in `cargo test`, because scenario jobs never count as coverage and mutants of the ports need
     them. Each has a per-test allowance that can only go down.

## 3. Enforcement

- **Measure `test_processes`.** This generalises `git_processes`.
  - `meta_run::Run` and `task::CommandLine` write one line per start to a directory named by `META_SPAWN_TRACE`,
    when it is set. git writes its own trace2 lines.
  - The xtask runner already runs one test per process. It gives each test a fresh directory for both variables.
  - `git_trace::GitProcesses` becomes `Spawns`, keyed by test and program.
  - Counts are integers: raw, unscaled, and the same on every runner.
  - An unreadable trace fails closed, as `GitProcesses::traced` does today.
- **CI.** The `git-count` job becomes `process-count`, with the same steps (`Execution::Counted`).
  It prints the per-test table and spends:
  - `test-processes`: total starts, allowance set at the landing measure;
  - a per-test rule: allowance 0 outside the port-contract list in `tooling/policy.toml`;
  - per contract test, a baseline row in `self/test-processes-baseline.toml`. It uses the `{ baseline = … }`
    mechanism of `lint-findings`, so a row can only go down.
- **Static guard.** Lint `process_starters` generalises `git_starters`. `std::process::Command` is allowed only in
  `meta-run`, `task::command_line` and port-contract tests. A test that starts a process another way is a lint
  finding, so the count cannot be bypassed.
- **Scenario job.** It has its own budgets: `scenario-wall-time` and `scenario-processes`. Each is set once from the
  landing measure (#186 measured 6 such tests at 0.21 s). Its tests do not count toward `test-cpu-time` or coverage.
- No budget number changes. `git-processes` (652) and `git-starters` (19) stay. Their retirement after the zero rule
  is the owner's decision.

## 4. The PR stack

The stack starts from main after this doc is accepted. Branches are `task-914f9a9748b07f03-N`:

```
gh stack init task-914f9a9748b07f03-2
gh stack add task-914f9a9748b07f03-3   # repeat for each PR below
gh stack submit
```

In every PR, the first commit sets the baseline rows of the tests in scope to 0. CI is then red on exactly those
tests. The next commit converts them. Each PR body gives the `process-count` table, 3 local runs, and CI CPU with
its runner reference.

| # | branch | scope | tests | processes removed (local) | CPU removed (local, measured cost) |
|---|---|---|---|---|---|
| 1 | -2 | Ledger: `META_SPAWN_TRACE`, `Spawns`, `test-processes` budget, baseline file, `process_starters` lint, `process-count` job. Report only. | 0 | 0 | 0 |
| 2 | -3 | meta-git reads: the 19 `GitArgv` calls move to git2. | project | 5, plus most git under `meta` (304 of 593 on CI) | 34 ms, plus part of every `it` test |
| 3 | -4 | Local transport and CAS in `meta_git::Remote`; xtask `Revision`, gate checkouts, mutation scope and shard plan on meta-git. | 20 | 274 | 1.73 s |
| 4 | -5 | `Call` and `Answer.then`; ambient-read lint; hooks pre-tool-use and session-start state, cli writes in process. | 3 | 28 | 0.25 s |
| 5 | -6 | gh `Recorded` source in process; status, gc and the replayer tests; fake gh scripts deleted. | 4 | 59 | 0.62 s (gc: CI 226 ms ÷ 0.52) |
| 6 | -7 | CI watcher in process on #200's verdicts; `it` ci becomes `Call` tests. | 6 | 136 | 0.85 s |
| 7 | -8 | Claim and post-tool-use in process. | 3 | 96 | 0.56 s |
| 8 | -9 | Scenario target and job (workflow from #186); scenarios 1-6; residue leaves `it`. | moves 6 | all left outside contracts | the residue |
| 9 | -10 | Zero rule: 0 outside the contract list; baseline rows only for contracts. | 0 | 0 | 0 |

PR 6 needs #200 on main. PRs 2-5 do not, so the stack starts now and PR 6 restacks with `gh stack sync`.
The "CPU removed" column is the measured cost of the converted tests. The in-process tests that replace them add back
about 10-25 ms each (section 6).

## 5. Earlier PRs

| PR | decision | why |
|---|---|---|
| #200 CI watcher decisions as pure functions | **keep**, land before PR 6 | It is the functional core that PR 6 needs. Mutants: same 3 missed as main. Fix its red `test-budget` lint (4 tests in `session_dir.rs`, 3 allowed) and merge main. |
| #194 signal mask table for `Ignored` | **keep**, land now | It is a small, green, pure table. Those 8 mutants need it whatever the stack does. |
| #191 in-suite CI status tests | **close** | #200 carries its `CiStatus` and recorded-check tests. Its session-start and hook tests are replaced by the `Call` tests in PRs 4 and 7. Its CI is red. |
| #186 CI watch scenarios in their own job | **close; fold the job** | It moved e2e tests out before in-process replacements existed: its proof run missed 96 mutants (main misses 3). PR 8 reuses its `test = false` target and `scenarios` job, after PRs 4-7 hold the coverage. |

## 6. Expected end state

Method: CPU after = measured CPU of what stays + an estimate for the converted tests.
- What stays: 372 non-spawning tests, 1.32 s; 17 port contracts, 0.42 s.
- Converted tests: about 40 tests at 10-25 ms local each, from the in-process tests above. That is 0.4-1.0 s.
- New verdict tests that replace mutant coverage: about +0.1 s.

| measure | now | after the stack |
|---|---|---|
| local test CPU | 5.52 s | 2.2-2.8 s |
| CI test CPU (local × 0.47-0.69, the range of the 3 latest 426-test runs) | 2.62-3.82 s | 1.0-2.0 s, at most 49 % of the 4.0 s line |
| CI spread over runner levels | 2.61-4.69 s, sd 0.52 s | about 1 s range, sd about 0.24 s (CPU ∝ reference, so the spread scales with the total) |
| processes started by tests | 752 local, 593 git on CI | 0 outside contracts; contracts 128, ratchet down |
| `it` tests that start `meta` | 14 | 0 (6 scenarios in their own job) |

Check: after PR 9, take 20 main runs and fit CPU against the runner reference, as in section 1. The target is
the slope × the highest reference seen (1.16 s) ≤ 2.4 s, which is 60 % of the line. If not, the next lever is the runner floor:
427 test processes at 1.1 ms or more each, about 0.25 s CI. Changing how per-test CPU is taken is an owner decision.

## Owner decisions

1. Path remotes go through libgit2 in process (section 2.2), and URLs with a scheme through the CLI.
2. The in-process `Recorded` gh source extends the replay decision.
3. Port-contract tests stay in `cargo test` with per-test allowances.
4. Retire `git-processes` and `git-starters` after the zero rule.
