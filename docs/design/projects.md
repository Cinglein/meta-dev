# Projects: design

Task: task-7b6e5d9baadb95f8. This document is a design only. It absorbs the feature kind of task-6d498ba4.

**Goal metric.** The goal is total spend per delivered project. Total spend includes own goals. An own goal is never a delivery.

**Kernel concepts.** A project uses only these concepts:
- Item (K2): a new item kind, `project`.
- Stage (K4): the project typestate (§1).
- Budget (K9): each acceptance measure is a budget row in `tooling/policy.toml` (§3).
- Signal (K5): readings from main, the no-progress flag and the regression flag (§3, §5).
- Edge (K3): `instruments`, `serves`, `caused_by` and the decision `blocks` edge (§4).

The new value types are `Purpose` and `Root`. There are no new scales, no new ledgers and no new journals.

**Root** (owner, 2026-10-30). Each project addresses one of two root concerns:
- `process`: enforcement. We bind ourselves to a better subset of actions than naive instinct.
- `state`: architecture. This is everything we built, both features and own goals.

Performance, cost and measurement are symptoms of these two roots. They are not a root.

**Measurement is never its own root or its own project** (owner, 2026-10-30: "everything needs to be observable and measured by default"). It is a default part of every acceptance. Each acceptance names the observability that measures the project: its budget rows, its spans and its status line. Until main produces all of them, only instrumentation work can be claimed (§1).

The Delivered guard is the pairing of the two roots: the measure on target is the state change, and the gating enforcement is the process that keeps it.

## 1. Typestate

`ProjectStage = Proposed | Accepted | Instrumented | Active | Delivered | Regressed`. A fold over meta/log gives the stage. `Active` is derived. It has no event.

| From | To | Event | Guard | Who |
|---|---|---|---|---|
| — | Proposed | `ProjectProposed` | The goal text is not empty. The division is known. | Any actor. |
| Proposed | Accepted | `ProjectAccepted` | The type requires a root, at least one acceptance row (budget id, measure, target), at least one span and at least one CI-side enforcement point (§3). None of them needs to exist yet. | Owner only. |
| Accepted | Instrumented | `ProjectObserved` | Main's ledger has a row for each acceptance budget, with the named measure and a measured spend (no `unmeasured`). Each named span is in main's push-run trace. | meta only (`meta project observe`). |
| Instrumented | Active | none (derived) | A `serves` task has a holder. | — |
| Active | Instrumented | none (derived) | No `serves` task has a holder. | — |
| Instrumented or Active | Accepted | `ProjectObserved` | A row became unmeasured or a span is missing on main, before delivery. | meta only. |
| Instrumented or Active | Delivered | `ProjectObserved` | Each acceptance row is on target on main. Each enforcement point is gating on main. The instrumentation guard still holds. The evidence names the main commit and its push run. | meta only. |
| Delivered | Regressed | `ProjectObserved` | A row is off target or unmeasured, a span is missing, or an enforcement point stops gating. | meta only. |
| Regressed | Delivered | `ProjectObserved` | The same guard as the first delivery. | meta only. |
| Any | Retired | `EntryRetired` (existing) | Proposed: a reason. Accepted and later: owner only. | Proposed: the lead or the owner. Later: owner only. |

**Which tasks attach and which can be claimed:**

| Stage | `instruments` | `serves` | `caused_by { project }` |
|---|---|---|---|
| Proposed | refused | refused | refused |
| Accepted | attach and claim | attach; claim refused | attach and claim |
| Instrumented, Active | attach and claim | attach and claim | attach and claim |
| Delivered | refused | refused | attach and claim |
| Regressed | refused | refused | attach and claim (the only way to fix it) |

Rules:
- A fix for a regression is an own goal (§4).
- `ProjectObserved` that does not change the stage is refused (`Refusal::Unchanged`). The log gets one record per transition, not one per poll.
- `EntryCorrected` changes the goal text. It cannot change the acceptance. A new acceptance needs a new `ProjectAccepted` from the owner, and only before delivery. It puts the project back to Accepted.

## 2. Schema change

New entity: `Project`, id prefix `project`.

New events:

| Event | Payload |
|---|---|
| `ProjectProposed` | `NewProject { project, text, division }` |
| `ProjectAccepted` | `Acceptance { project, root: Root, rows: Vec1<AcceptanceRow>, spans: Vec1<SpanName>, enforcement: Gating }` (`Gating` is a `Vec1<Enforcement>` with at least one `Budget` or `Step` point) |
| `ProjectObserved` | `Observation { project, main: CommitId, run: RunId, verdict: Verdict }` |
| `TaskPurposed` | `TaskPurpose { task, purpose: Purpose }` (attach an existing task) |

Changed event: `TaskOpened` gains `purpose: Option<Purpose>`, with `#[serde(default, skip_serializing_if = "Option::is_none")]`.

```
Purpose      = Instruments(Id<Project>)
             | Serves(Id<Project>)
             | CausedBy { cause: Cause, project: Option<Id<Project>> }
Cause        = Pull(PullRequest) | Item(Target) | Task(Id<Task>)
Root         = Process | State
AcceptanceRow = { budget: BudgetId, measure: MeasureName, target: Amount }
Verdict      = Instrumented | Uninstrumented(Gap) | OnTarget
             | OffTarget(BudgetId) | NotGating(Enforcement)
Gap          = Unmeasured(BudgetId) | NoSpan(SpanName)
```

- **Instrumentation is a purpose.** `Instruments(p)` is typed. It is not a word in a task title. Its tasks add the measure, the spans and the budget rows.
- **Root** is set at Accepted, not at Proposed. A Proposed project has no root, so its type has no `root` field.
- **Rows.** `MeasureName` is the policy spelling of a `Measure`. `meta project observe` parses it against `Measure`. `Amount`, `BudgetId` and `MeasureName` move into the crate `meta-budget`, so `tooling/task` and the log use one budget vocabulary. `MeasureName` names only simple measures (one snake-case word, for example `test_line_share`). If a project needs a measure that takes parameters, `MeasureName` widens before PR 6.
- **Acceptance types.** `AcceptanceRow`, the enforcement points, the span names and `Verdict` are in the crate `meta-acceptance`, below `meta-schema`. Two rows for the same budget are refused.
- **Spans.** `spans` names the trace spans that the project's output emits. In the log a span is a `SpanName` (one trimmed line, for example `command claim`), not the `meta-trace` enum, so trace spellings never become part of the log format. `meta project observe` maps a `SpanName` to a `SpanKind` on the `meta-trace` side. An instrumentation task may add a `SpanKind` variant. A missing trace on main fails closed.
- **One purpose per task.** `Purpose` is one enum, so a task cannot both serve and be an own goal. The `project` in `CausedBy` only places the task under a project in status. Its spend stays an own goal.
- **A purpose is set once.** A task gets its purpose at `TaskOpened` or by one `TaskPurposed`. The fold (PR 2) refuses a second `TaskPurposed`, and a `TaskPurposed` for a task that opened with a purpose.

**Version.** Each schema version is a named constant made from the version before it (`FIRST_KEEPING_HOLDERS`, then `PROJECTS`), and `SchemaVersion::current()` returns the latest. Two PRs that each add a version then conflict in the text, so they cannot merge into one number by accident. PR 1 adds `PROJECTS`. An older build reads the new marker and stops writing, as it does today.

**Old logs.**
- Old events are unchanged and still read.
- An old `TaskOpened` has no purpose. It reads as `Unlinked`. Its spend counts as an own goal (fail closed).
- `PROJECTS` does not require a purpose. Until the CLI can supply one (PR 3), every `meta open` writes a `TaskOpened` with no purpose.
- `SchemaVersion::requires_purpose()` starts at its own later version, `PURPOSES`, which lands with or after the CLI that supplies a purpose. After that marker, the fold refuses a `TaskOpened` with no purpose. This uses the same pattern as `keeps_holders_on_reopen`.
- Open tasks from before `PURPOSES` get a purpose through `TaskPurposed`.

**Selector names** (owner, 2026-10-30). Design §2's selector list becomes:

```
Selector = Landing(pr) | Stage(task, StageKind) | Task(task) | Footprint(crates)
         | Division(d) | Repo | AllRepos | Project(Id<Project>) | Nothing{ revisit: Trigger }
```

- `repo` (`Repo`): one repo.
- `all-repos` (`AllRepos`): the whole repo set. Design §2 called it `Project`.
- `Project(Id<Project>)`: one project of this item kind.

## 3. Measures and enforcement points

**Acceptance row.** The owner names a budget id, its measure and a target. A row is **on target** when main's measured spend is at or below the target.
- The target is an owner decision, like every budget number (design §2).
- A new measure is a new `Measure` variant with its own budget row. An instrumentation task adds both. The row starts with `overspend = report`.
- The project holds the target. The budget row holds the enforcement lines. They are different numbers with different jobs.

**Readings on main.** The push run on main writes `ledger.json` as a CI artifact. It holds:
- `rows`: every budget row (id, measure, spend, lines, state, overspend), from `Ledger::statement`.
- `gates`: every enforcement point that is live on this commit (see below).

The push run also uploads its trace (task-433270d3). Caps (design §7) read the same artifact. A missing artifact, a missing row, an unmeasured spend or a missing trace fails closed.

**Enforcement point.**

```
Enforcement = Budget(BudgetId)   -- gating: the row has overspend = block and window = per_change
            | Step(PointName)    -- gating: the step ran in the push run, and ci-gate needs its job
            | Hook(PointName)    -- gating: plugin/hooks.json on main registers it
```

- At least one point must be `Budget` or `Step`. Hooks guard against honest mistakes only (threat model). A required CI check is the backstop.
- "Merged and gating" is one check: the point is in the `gates` list of main's latest push-run artifact. A point can only be there after its PR merged.
- A per-change budget blocks at its blocking line. If that line is above the target, drift between the two raises `Regressed`, not a CI failure. To make CI hold the target, the owner can set the allowance to the target at delivery. That is a budget-number decision.

## 4. Spend attribution and own goals

**Spend.** The spend ledger (task-41cdd344) gives spend per task: weighted tokens per model and wall time. This design adds nothing to it.

**Attribution.**

| Task purpose | Spend goes to |
|---|---|
| `Instruments(p)` | project `p` |
| `Serves(p)` | project `p` |
| `CausedBy { .. }` | own goal |
| `Unlinked` (old task) | own goal |
| no task (for example lead orchestration) | own goal |

The last two rows make the measure fail closed. Unlinked spend is visible, so the cure is to link it.

**Goal metric.**
- `goal = total spend in window / projects delivered in window`.
- A project that is Regressed at the end of the window does not count.
- Zero deliveries gives `unmeasured`, not infinity.

**Own-goal detection.**
- **Explicit.** `meta open --caused-by <pr|item|task>`. The CLI checks that a `Pull` cause is a merged PR in this repo.
- **Derived (later PR).** Triage writes `caused_by` without a human:
  - a failure that the batch queue attributes to a landed PR gives `CausedBy(Pull(that PR))`;
  - a revert PR gives `CausedBy(Pull(the reverted PR))`;
  - a task for a Regressed project gives `CausedBy(Pull(the first main commit's PR that read off target))`.
- `caused_by` chains feed the elegance lint as architecture signals (task-6d498ba4).

## 5. Status rendering

`meta status` shows projects first, then the own-goal line and the goal line, then flags and the division tree.

```
projects:
  project-1a2b tests below warn       Active        state    test-time 5.9s>4.8s  test-spawns 31>0   gate budget:test-time ok, step:test ok    spend 41.2M  [no progress 9.0M]
  project-3c4d dogfooding gate        Accepted      process  needs: unmeasured direct-merges, no span Command(queue)   instrumenting 1 task   spend 3.1M
  project-5e6f elegance lint gating   Instrumented  process  effect-share 31%>25%   gate budget:effect-share MISSING   spend 0.4M
  project-7a8b waterfalls             Delivered     process  trace-unlinked-waits 0<=0   gate budget:trace-unlinked-waits ok   spend 18.0M
  project-9c0d spec claims            Proposed      —
own goals: 22.4M of 84.7M (26%): caused_by 15.0M, unlinked 7.4M
goal: 84.7M / 1 delivered (7d)
```

- Each project shows its stage, then its root once it is Accepted.
- An Accepted project shows `needs:`, which lists each gap that holds it back from Instrumented, and its open instrumentation tasks.
- From Instrumented on, each acceptance row shows `now` against its target, from main's latest artifact.
- Each enforcement point shows `ok` (gating) or `MISSING`.
- Spend shows `spend unmeasured` until the spend ledger lands (the existing `EffortSpend::Unmeasured`).
- Tasks in the tree show their purpose: `instruments project-3c4d`, `> project-1a2b` or `caused_by #217`.

**New flags.** Both are signals (K5), rated by the one rating rule (design §1.3).

| Flag | Measure | Line |
|---|---|---|
| `NoProgress(project)` | Spend on the project since the last improvement of any acceptance row on main. In Accepted: spend since the last closed gap. | `[projects] no_progress` in policy (owner sets). |
| `Regressed(project)` | The stage is Regressed. | Always raised. |

"Spend since the last improvement" needs no time window. It reads main's push-run artifacts through the GitHub service cache.

## 6. Enforcement of each stage

| Stage or rule | Where it is enforced |
|---|---|
| Proposed: no tasks attach | Fold: every purpose that names a Proposed project is refused. |
| Accepted: owner only | Fold: `ProjectAccepted` from an actor other than `owner` is refused. CI owner-event check (design §2) covers the budget rows that instrumentation adds. |
| Accepted: only instrumentation is claimed | `meta claim` refuses a `serves` task while its project is Accepted. The refusal names the gaps and the open `instruments` tasks. CI `TrackedWork` adds `Untracked::Uninstrumented(project)` for a PR of such a task. |
| Instrumented: measured on main | `meta project observe` writes `Instrumented` only when every row is measured and every span is in the trace. A gap before delivery writes `Uninstrumented` and claims of `serves` tasks stop again. |
| Every task has a purpose | Fold (from `PURPOSES`). `meta claim` refuses an `Unlinked` task and names `meta serve`, `meta instrument` or `meta cause`. CI `TrackedWork` adds `Untracked::Unpurposed(task)`. |
| Regressed: fixes are own goals | Fold: `instruments` and `serves` to a Regressed project are refused. The task needs `CausedBy { project: Some(p) }`. |
| Delivered: on target and gating | `meta project observe` reads the artifact and the trace and writes `ProjectObserved` only on a passing guard. The `Observation` type requires the commit and run as evidence. A CI audit on push to main replays each `ProjectObserved` against its named run artifact and fails closed on a mismatch. |
| Landing | `meta queue run` refuses a PR whose task is `Unlinked`. After each landing, it runs `meta project observe` for the projects that the landed tasks instrument or serve. |
| Regressed: raised | `meta heartbeat tick` runs `meta project observe` for every Instrumented, Active and Delivered project. |
| Decisions | A decision blocks a project with `Blocks { by: Decision(d), selector: Project(p) }`. `meta claim` refuses a task with a purpose that names `p` while the decision is open. |

## 7. PR plan

Each PR is small and comes off main. Each first commit is red first.

1. **Schema.** `Project` entity, `Purpose` (with `Instruments`), `Cause`, `Root`, `AcceptanceRow`, `Verdict`, four events, `TaskOpened.purpose`, `meta-budget` and `meta-acceptance`, version `PROJECTS`. Red: round-trip and old-log rows fail.
2. **Fold.** `ProjectState` typestate with `Instrumented`, the attach table in §1, the refusals in §6, and the refusal of a second purpose. Red: a test that lists every refused transition and attach.
3. **CLI.** `meta project propose|accept|list`, `meta open --instruments|--serves|--caused-by`, `meta instrument`, `meta serve`, `meta cause`. `meta claim` refuses `Unlinked`, and refuses `serves` before Instrumented. Goldens re-recorded. Version `PURPOSES` and `requires_purpose` land here or later, not before the CLI can supply a purpose.
4. **CI.** `TrackedWork` refuses `Unpurposed` and `Uninstrumented`. `meta queue run` refuses `Unlinked`.
5. **Ledger artifact.** `xtask ci` writes `ledger.json` with `rows` and `gates` on push to main; the workflow uploads it. Shared with caps (design §7).
6. **Observe.** `meta project observe` reads the artifact and the push run's trace (task-433270d3 uploads it) through meta-gh and writes `ProjectObserved`: `Instrumented`, `Uninstrumented`, delivery and regression. Heartbeat and queue call it. The push-to-main audit replays observations.
7. **Status.** Projects section with `needs:` for Accepted projects, own-goal line, goal line, `NoProgress` and `Regressed` flags. Spend reads `Unmeasured` until task-41cdd344 lands.
8. **Derived causes.** Triage writes `caused_by` from batch attribution, reverts and regressions. Waits on the batch queue (design §9).
8a. **Retire a project.** `Entry::Project`, so that `EntryRetired` and `EntryCorrected` can name a project, with the retirement guards in §1. This needs its own schema version, because older builds cannot parse a project entry. It lands before PR 9.
9. **Seed.** No code. The lead writes `ProjectProposed` for §8, the owner writes `ProjectAccepted`, and the lead opens the instrumentation tasks first and attaches the open tasks with `TaskPurposed`.

Order: 1, 2, 3, then 4, 5 and 7 in parallel, then 6, then 8. 8a lands before 9. The owner accepted this design on 2026-10-30. PR 1 starts after the hook PRs land. The lead retires task-6d498ba4 into this task after PR 1.

## 8. First projects

The four projects start at Accepted. Each one has its instrumentation tasks first. Its serving tasks attach at once, but they can be claimed only after the project is Instrumented. The targets below are proposals. Each number is the owner's decision.

| Project | Root | Acceptance rows (budget: measure, target) | Spans | Instrumentation tasks (first) | Enforcement points | Serving tasks |
|---|---|---|---|---|---|---|
| **Tests below warn, no stray spawns** | `state` (test architecture) | Existing `test-time`, `test-cpu-time`, `slowest-test`, `git-processes`, each with its current warning line as the target. New `test-spawns`: `test_spawns_outside_allowance`, target 0. | `Spawn(Program)` for each spawn in a test. | Add the `test_spawns_outside_allowance` measure, per-test spawn spans and the `test-spawns` row. | `Budget(test-spawns)` blocking per change; `Step(Test)`. | task-914f9a97 (test architecture) and its stack. |
| **Dogfooding gate** | `process` | New `unleased-agent-starts`: agent starts with no live lease from `meta claim --agent`, target 0. New `direct-merges`: main commits with no queue landing record, target 0. | `Command(claim)`, `Command(hook)`, `Command(queue)`. | Add both measures (from the hook record and the push-to-main audit), the spans and both rows. | `Hook(pre-tool-use agent)` (task-8065b583, PR #226); `Budget(direct-merges)` blocking, audited on push to main. | task-8065b583, task-a940d57c, task-71d90f14, task-cccccfaa, task-a3df18c7 (retire the stopgap at delivery). |
| **Elegance lint gating** | `process` (a process that guards state) | New rows on the existing measures `effect_share`, `effect_dispersion`, `repeated_shapes`, and a parallel-vocabulary count (B26). Targets: the owner sets them from the first readings. | A span for the xtask lint step (adds a `SpanKind` variant for xtask steps). | Add the B26 measure, the xtask step span and the four rows. | `Budget(effect-share)`, `Budget(effect-dispersion)`, `Budget(repeated-shapes)` with overspend = block; `Step(Lint)`. | task-3b34f81b, task-4b4e69c3, task-e7d5c91c, task-e2574ad0. |
| **Dependency-aware waterfalls** | `process` (owner, 2026-10-30): measure, then optimise. Known-expensive work gets spans automatically. Other work is optimised only after a waterfall shows it is expensive; then it is profiled and given spans. The waterfall tool is state that serves this process. | New `trace-unlinked-waits`: waits on the CI trace critical path with no `follows_from` edge, target 0. New `trace-untraced-share`: untraced share of the critical path; the owner sets the target. | `Command(Verb)`, `Spawn(git)`, `Network(gh)` in the uploaded CI trace. | task-433270d3 (CI uploads one trace per run), task-b6a65742 (git spans at the port), and both rows. | `Budget(trace-unlinked-waits)` and `Budget(trace-untraced-share)` blocking per change on the uploaded CI trace. | task-9da916d3. |

Each new budget row starts with `overspend = report`. A row with `report` is not gating, so a project cannot reach Delivered until the owner switches the row to `block`.

## 9. Open questions

- Naming: formal rules for names are a separate, deferred task (owner, 2026-10-30).

## 10. Later: roots as the primary structure

**Deferred.** Do not start this work until the current fires are out: the hook stack, the test stack, and projects PRs 1-9.

- **Divisions.** The divisions change into two root divisions, `process` and `state`.
- **Measurement.** Measurement becomes a layer that every project and every division reads. It is not a division.
- **Diagnosis before close.** A signal cannot close until it is diagnosed down to a root. This applies to every signal: a rated issue, an own goal or an incident. The diagnosis names a process gap or a state flaw. `caused_by` chains are the evidence.
- **Absorbs** the deferred architectural-concern typestate, task-f105a220.
- **Status.** This connects to the `meta status` dogfooding enforcement: status shows each signal that has no root yet.
