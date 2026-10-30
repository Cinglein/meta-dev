# Projects: design

Task: task-7b6e5d9baadb95f8. This document is a design only. It absorbs the feature kind of task-6d498ba4.

**Goal metric.** The goal is total spend per delivered project. Total spend includes own goals. An own goal is never a delivery.

**Kernel concepts.** A project uses only these concepts:
- Item (K2): a new item kind, `project`.
- Stage (K4): the project typestate (§1).
- Budget (K9): the acceptance measure is a budget row in `tooling/policy.toml` (§3).
- Signal (K5): readings from main, the no-progress flag and the regression flag (§3, §5).
- Edge (K3): `serves`, `caused_by` and the decision `blocks` edge (§4).

The new value types are `Purpose` and `Root`. There are no new scales, no new ledgers and no new journals.

**Root** (owner, 2026-10-30). Each project addresses one of two root concerns:
- `process`: enforcement. We bind ourselves to a better subset of actions than naive instinct.
- `state`: architecture. This is everything we built, both features and own goals.

Performance, cost and measurement are symptoms of these two roots. They are not a root.

**Measurement is never its own root or its own project** (owner, 2026-10-30: "everything needs to be observable and measured by default"). It is a default part of every acceptance. Each acceptance names the observability that measures the project: its budget rows, its spans and its status line. If what a project builds cannot be observed, the project cannot be accepted.

The Delivered guard is the pairing of the two roots: the measure on target is the state change, and the gating enforcement is the process that keeps it.

## 1. Typestate

`ProjectStage = Proposed | Accepted | Active | Delivered | Regressed`. A fold over meta/log gives the stage. `Active` is derived. It has no event.

| From | To | Event | Guard | Who |
|---|---|---|---|---|
| — | Proposed | `ProjectProposed` | The goal text is not empty. The division is known. | Any actor. |
| Proposed | Accepted | `ProjectAccepted` | The CLI checks that each acceptance budget id is in `tooling/policy.toml` on main. The type requires a root and at least one CI-side enforcement point (§3). **Observable:** the type requires at least one span. The CLI checks that each acceptance budget has a `Measure` that the CI ledger produces, so its row can never read `unmeasured` by design. The projects section of status shows every Accepted project, so the status line needs no field. | Owner only. |
| Accepted | Active | none (derived) | A task that serves the project has a holder. | — |
| Active | Accepted | none (derived) | No task that serves the project has a holder. | — |
| Accepted or Active | Delivered | `ProjectObserved` | Every acceptance row is measured and on target on main. Every named span is in the push run's trace. Every enforcement point is gating on main. The evidence names the main commit and its push run. | meta only (`meta project observe`). |
| Delivered | Regressed | `ProjectObserved` | An acceptance row is off target on main, or an enforcement point stops gating. | meta only. |
| Regressed | Delivered | `ProjectObserved` | The same guard as the first delivery. | meta only. |
| Any | Retired | `EntryRetired` (existing) | Proposed: a reason. Accepted and later: owner only. | Proposed: the lead or the owner. Later: owner only. |

Rules:
- In `Proposed`, no task can attach. The fold refuses `serves` to a Proposed project.
- In `Delivered`, new `serves` is refused. New work on a delivered goal is a new project.
- In `Regressed`, new tasks for the project must carry `caused_by` (§4). A fix for a regression is an own goal.
- `ProjectObserved` that does not change the stage is refused (`Refusal::Unchanged`). The log gets one record per transition, not one per poll.
- `EntryCorrected` changes the goal text. It cannot change the acceptance. A new acceptance needs a new `ProjectAccepted` from the owner, and only before delivery.

## 2. Schema change

New entity: `Project`, id prefix `project`.

New events:

| Event | Payload |
|---|---|
| `ProjectProposed` | `NewProject { project, text, division }` |
| `ProjectAccepted` | `Acceptance { project, root: Root, target: Vec1<BudgetId>, spans: Vec1<SpanKind>, enforcement: Vec1<Enforcement> }` |
| `ProjectObserved` | `Observation { project, main: CommitId, run: RunId, verdict: OnTarget \| OffTarget(BudgetId) \| NotGating(Enforcement) }` |
| `TaskPurposed` | `TaskPurpose { task, purpose: Purpose }` (attach an existing task) |

Changed event: `TaskOpened` gains `purpose: Option<Purpose>`, with `#[serde(default, skip_serializing_if = "Option::is_none")]`.

```
Purpose = Serves(Id<Project>)
        | CausedBy { cause: Cause, project: Option<Id<Project>> }
Cause   = Pull(PullRequest) | Item(Target) | Task(Id<Task>)
Root    = Process | State
```

`root` is set at Accepted, not at Proposed. The owner decides it with the measure. A Proposed project has no root, so its type has no `root` field. A new root needs a new `ProjectAccepted` before delivery, as for the measure.

`spans` names the trace spans that the project's output emits, such as a hook, a CI step or a command. `SpanKind` is the existing `meta-trace` type (`Command(Verb) | Spawn(Program) | Network(Program) | Fs`). A missing trace on main fails closed, so the project cannot be delivered.

`Purpose` is one enum, so a task cannot both serve and be an own goal. The `project` in `CausedBy` only places the task under a project in status. Its spend stays an own goal.

**Version.** `SchemaVersion::current()` goes from 7 to 8. An older build reads the version 8 marker and stops writing, as it does today.

**Old logs.**
- Old events are unchanged and still read.
- An old `TaskOpened` has no purpose. It reads as `Unlinked`. Its spend counts as an own goal (fail closed).
- `SchemaVersion::requires_purpose()` is true from version 8. After the marker, the fold refuses a `TaskOpened` with no purpose. This uses the same pattern as `keeps_holders_on_reopen`.
- Open tasks from before version 8 get a purpose through `TaskPurposed`.

**Selector names** (owner, 2026-10-30). Design §2's selector list becomes:

```
Selector = Landing(pr) | Stage(task, StageKind) | Task(task) | Footprint(crates)
         | Division(d) | Repo | AllRepos | Project(Id<Project>) | Nothing{ revisit: Trigger }
```

- `repo` (`Repo`): one repo.
- `all-repos` (`AllRepos`): the whole repo set. Design §2 called it `Project`.
- `Project(Id<Project>)`: one project of this item kind.

## 3. Measures and enforcement points

**Acceptance measure.** The target is a list of budget ids in `tooling/policy.toml`. A row is **on target** when its `budgets::State` is `Ok`, which is at or below its warning line.
- The owner sets the number, as for every budget. A budget number is an owner decision (design §2).
- A new measure is a new `Measure` variant with its own budget row. It is not a project field.
- A project does not hold a number. The budget holds it. There is one copy.

**Readings on main.** The push run on main writes `ledger.json` as a CI artifact. It holds:
- `rows`: every budget row (id, measure, spend, lines, state, overspend), from `Ledger::statement`.
- `gates`: every enforcement point that is live on this commit (see below).

Caps (design §7) read the same artifact. A missing artifact, a missing row or an unmeasured spend fails closed: no delivery, and status shows `unmeasured`.

**Enforcement point.**

```
Enforcement = Budget(BudgetId)   -- gating: the row has overspend = block and window = per_change
            | Step(CiStep)       -- gating: the step ran in the push run, and ci-gate needs its job
            | Hook(HookName)     -- gating: plugin/hooks.json on main registers it
```

- At least one point must be `Budget` or `Step`. Hooks guard against honest mistakes only (threat model). A required CI check is the backstop.
- "Merged and gating" is one check: the point is in the `gates` list of main's latest push-run artifact. A point can only be there after its PR merged.
- A per-change budget blocks at its blocking line. The target is the warning line. Drift between the two raises `Regressed`, not a CI failure. To make CI hold the target, the owner can lower the allowance at delivery. That is a budget-number decision.

## 4. Spend attribution and own goals

**Spend.** The spend ledger (task-41cdd344) gives spend per task: weighted tokens per model and wall time. This design adds nothing to it.

**Attribution.**

| Task purpose | Spend goes to |
|---|---|
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
  project-1a2b tests below warn       Active    state    test-time 5.9s>4.8s  test-spawns 31>0   gate budget:test-time ok, step:test ok    spend 41.2M  [no progress 9.0M]
  project-3c4d dogfooding gate        Accepted  process  unlinked-spawns 4>0  direct-merges 2>0   gate hook:agent-lease ok, budget:direct-merges MISSING  spend 3.1M
  project-5e6f elegance lint gating   Proposed  —
  project-7a8b waterfalls             Delivered process  untraced-share 6%<=10%                   gate budget:untraced-share ok              spend 18.0M
own goals: 22.4M of 84.7M (26%): caused_by 15.0M, unlinked 7.4M
goal: 84.7M / 1 delivered (7d)
```

- Each project shows its root (`process` or `state`) after its stage, once it is Accepted.
- Each acceptance row shows `now` against its warning line, from main's latest artifact.
- Each enforcement point shows `ok` (gating) or `MISSING`.
- Spend shows `spend unmeasured` until the spend ledger lands (the existing `EffortSpend::Unmeasured`).
- Tasks in the tree show their purpose: `> project-1a2b` or `caused_by #217`.

**New flags.** Both are signals (K5), rated by the one rating rule (design §1.3).

| Flag | Measure | Line |
|---|---|---|
| `NoProgress(project)` | Spend on the project since the last improvement of any acceptance row on main. | `[projects] no_progress` in policy (owner sets). |
| `Regressed(project)` | The stage is Regressed. | Always raised. |

"Spend since the last improvement" needs no time window. It reads main's push-run artifacts through the GitHub service cache.

## 6. Enforcement of each stage

| Stage or rule | Where it is enforced |
|---|---|
| Proposed: no tasks attach | Fold: `serves` to a Proposed project is refused. |
| Accepted: owner only | Fold: `ProjectAccepted` from an actor other than `owner` is refused. CI owner-event check (design §2) covers the budget rows it adds. |
| Every task has a purpose | Fold (version 8). `meta claim` refuses an `Unlinked` task and names `meta serve` or `meta cause`. CI `TrackedWork` adds `Untracked::Unpurposed(task)`. |
| Regressed: fixes are own goals | Fold: `serves` to a Regressed project is refused. The task needs `CausedBy { project: Some(p) }`. |
| Delivered: on target and gating | `meta project observe` reads the artifact and writes `ProjectObserved` only on a passing guard. The `Observation` type requires the commit and run as evidence. A CI audit on push to main replays each `ProjectObserved` against its named run artifact and fails closed on a mismatch. |
| Landing | `meta queue run` refuses a PR whose task is `Unlinked`. After each landing, it runs `meta project observe` for the projects that the landed tasks serve. |
| Regressed: raised | `meta heartbeat tick` runs `meta project observe` for every Delivered project. |
| Decisions | A decision blocks a project with `Blocks { by: Decision(d), selector: Project(p) }`. `meta claim` refuses a task that serves `p` while the decision is open. |

## 7. PR plan

Each PR is small and comes off main. Each first commit is red first.

1. **Schema.** `Project` entity, `Purpose`, `Cause`, `Root`, four events, `TaskOpened.purpose`, version 8. Red: round-trip and old-log rows fail.
2. **Fold.** `ProjectState` typestate, the refusals in §1 and §6, `requires_purpose`. Red: a test that lists every refused transition.
3. **CLI.** `meta project propose|accept|list`, `meta open --serves|--caused-by`, `meta serve`, `meta cause`. `meta claim` refuses `Unlinked`. Goldens re-recorded.
4. **CI.** `TrackedWork` refuses `Unpurposed`. `meta queue run` refuses `Unlinked`.
5. **Ledger artifact.** `xtask ci` writes `ledger.json` with `rows` and `gates` on push to main; the workflow uploads it. Shared with caps (design §7).
6. **Observe.** `meta project observe` reads the artifact and the push run's trace (task-433270d3 uploads it) through meta-gh and writes `ProjectObserved`. Heartbeat and queue call it. The push-to-main audit replays observations.
7. **Status.** Projects section, own-goal line, goal line, `NoProgress` and `Regressed` flags. Spend reads `Unmeasured` until task-41cdd344 lands.
8. **Derived causes.** Triage writes `caused_by` from batch attribution, reverts and regressions. Waits on the batch queue (design §9).
9. **Seed.** No code. The lead writes `ProjectProposed` for §8 and attaches the open tasks with `TaskPurposed`. The owner accepts.

Order: 1, 2, 3, then 4, 5 and 7 in parallel, then 6, then 8. The owner accepted this design on 2026-10-30. PR 1 starts after the hook PRs land. The lead retires task-6d498ba4 into this task after PR 1.

## 8. First projects for the owner to accept

The targets below are proposals. Each number is the owner's decision.

| Project | Root | Acceptance budgets (on target = at or below warning) | Enforcement points | Serving tasks |
|---|---|---|---|---|
| **Tests below warn, no stray spawns** | `state` (test architecture) | Existing `test-time`, `test-cpu-time`, `slowest-test`, `git-processes`. New `test-spawns` (measure `test_spawns_outside_allowance`, count; proposed allowance 0, so the warning line is 0). | `Budget(test-spawns)` blocking per change; `Step(Test)`. | task-914f9a97 (test architecture) and its stack. |
| **Dogfooding gate** | `process` | New `unleased-agent-starts` (agent starts with no live lease from `meta claim --agent`, count over the window; proposed 0). New `direct-merges` (main commits with no queue landing record, count; proposed 0). | `Hook(pre-tool-use agent)` (task-8065b583, PR #226); `Budget(direct-merges)` blocking, audited on push to main. | task-8065b583, task-a940d57c, task-71d90f14, task-cccccfaa, task-a3df18c7 (retire the stopgap at delivery). |
| **Elegance lint gating** | `process` (a process that guards state) | New per-change budgets on the existing measures `effect_share`, `effect_dispersion`, `repeated_shapes`, and a parallel-vocabulary count (B26). Allowances: the owner sets them from today's readings. | `Budget(effect-share)`, `Budget(effect-dispersion)`, `Budget(repeated-shapes)` with overspend = block; `Step(Lint)`. | task-3b34f81b, task-4b4e69c3, task-e7d5c91c, task-e2574ad0. |
| **Dependency-aware waterfalls** | `process` (owner, 2026-10-30): measure, then optimise. Known-expensive work gets spans automatically. Other work is optimised only after a waterfall shows it is expensive; then it is profiled and given spans. The waterfall tool is state that serves this process. | New `trace-unlinked-waits` (waits on the CI trace critical path with no `follows_from` edge, count; proposed 0). New `trace-untraced-share` (untraced share of the critical path; the owner sets the line). | `Budget(trace-unlinked-waits)` and `Budget(trace-untraced-share)` blocking per change on the uploaded CI trace. | task-9da916d3, task-433270d3, task-b6a65742. |

Each new budget row is one owner decision. It lands in its project's first serving PR, with `overspend = report` until the owner accepts the numbers. A row with `report` is not gating, so the project cannot reach Delivered until the owner switches it to `block`.

## 9. Open questions

- Naming: formal rules for names are a separate, deferred task (owner, 2026-10-30).
- Measure before acceptance: the Accepted guard needs each acceptance `Measure` to exist on main. A Proposed project accepts no tasks. So who builds a new measure? Option: a Proposed project accepts one kind of task, which adds its measure and a `report` budget row, and that task's spend goes to the project.

## 10. Later: roots as the primary structure

**Deferred.** Do not start this work until the current fires are out: the hook stack, the test stack, and projects PRs 1-9.

- **Divisions.** The divisions change into two root divisions, `process` and `state`.
- **Measurement.** Measurement becomes a layer that every project and every division reads. It is not a division.
- **Diagnosis before close.** A signal cannot close until it is diagnosed down to a root. This applies to every signal: a rated issue, an own goal or an incident. The diagnosis names a process gap or a state flaw. `caused_by` chains are the evidence.
- **Absorbs** the deferred architectural-concern typestate, task-f105a220.
- **Status.** This connects to the `meta status` dogfooding enforcement: status shows each signal that has no root yet.
