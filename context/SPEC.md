# Meta-development repo: spec (draft, started 2026-10-22)

Sections down to the next top-level heading are rendered from the rule records in `self/spec`; edit those, then run `cargo run -p meta-spec`. Each rule shows its id, division, ladder rung and enforcer.

## Budgets (owner, 2026-10-26)
- `budget-definition` (architecture-and-planning, rung 1, no enforcer): A budget is a typed definition with an id, in tooling/policy.toml. Spend events in the log point at it.
- `budget-unit` (architecture-and-planning, rung 1, no enforcer): Unit, exactly one per budget: Count, Duration, Share (later maybe Money or CI minutes).
- `budget-window` (architecture-and-planning, rung 1, no enforcer): Window: per change (ratchet against the merge base), rolling per period, or standing (open items at any time).
- `budget-overspend` (architecture-and-planning, rung 1, no enforcer): Overspend action is part of the definition: block (default) or escalate to DISCUSS.
- `budget-measured-spend` (architecture-and-planning, rung 1, no enforcer): Spend is recorded by measurement only (CI and tools). Severity is the judged exception, checked later by calibration.
- `budget-raise` (architecture-and-planning, rung 1, no enforcer): Raising a budget is owner-only.
- `budget-waivers` (architecture-and-planning, rung 1, no enforcer): Waivers are spend from the waiver budget of the division whose rule they bend. They replace raw lint baselines.
- `budget-enforce-ci` (architecture-and-planning, rung 1, no enforcer): Budgets are enforced in CI (the local gate where Actions can't run).
- `budget-enforce-per-change` (architecture-and-planning, rung 1, no enforcer): Per-change budgets: measured fresh by CI.
- `budget-enforce-standing` (architecture-and-planning, rung 1, no enforcer): Standing budgets are derived from the log (open Criticals, waivers in force, concern backlog): CI folds meta/log read-only on every run. An over-budget standing budget blocks only PRs in its division, but a division whose property all code affects (for example CI performance) blocks every PR that could affect it.
- `budget-enforce-standing-base` (architecture-and-planning, rung 1, no enforcer): Standing budgets are enforced from the merge-base policy; the PR's own policy only has to parse and validate. A cap that the PR adds or tightens takes effect from the next PR. A cap that the PR loosens or drops still binds the PR itself, because the merge-base cap applies. The budget table marks a new cap as enforced from the next change. The merge base is the one with origin's default branch (origin/HEAD), never a branch that the PR names; a base policy that git cannot read fails the run, while a base without the file means no standing budgets. A PR whose task addresses an open item at the cap's level or more severe is paid down for that cap. A run that is not a pull request (a push to main) reports standing budgets in the table, marks the over-cap rows, and never fails: the debt rule blocks PRs, it does not turn main red.
- `budget-enforce-per-task` (architecture-and-planning, rung 1, no enforcer): Per-task budgets (effort, due): CI finds the PR's task via TaskLinked or the branch, and fails on overspend with no recorded extension; a PR with no task fails.
- `budget-enforce-spend-source` (architecture-and-planning, rung 1, no enforcer): Spend comes from local measurement (heartbeat from transcripts) written to the log; CI trusts it under the honest-mistakes threat model.
- `budget-enforce-machine-timing` (architecture-and-planning, rung 1, no enforcer): Machine budgets (CPU, memory, agents, disk) are runtime controls enforced by hooks and leases, not CI. Timing budgets are measured only on CI or in the local gate's clean run.

## Divisions (owner, 2026-10-26)
- `division-is-a-property` (architecture-and-planning, rung 1, no enforcer): A division is any behaviour or invariant made first class (e.g. Basho navigation). It is a named property, not a code area: divisions overlap freely in code and are isolated by tracking (own items, budgets, checks, status).
- `division-context` (architecture-and-planning, rung 1, no enforcer): An agent on a division sees only that division's context.
- `division-nesting` (architecture-and-planning, rung 1, no enforcer): Divisions nest arbitrarily. A child gets a subset of its parent's budget.
- `division-membership` (architecture-and-planning, rung 1, no enforcer): Every concern, issue and task belongs to exactly one division. An unplaced item belongs to the project root; growth there signals a missing division.
- `division-top-level` (architecture-and-planning, rung 1, no enforcer): Top level, in every project: (1) features and maintenance (navigation, the tracker, guarding lints and mutants); (2) performance and cost (CI and test time, mutation time, Actions minutes and money, disk, agent wall time); (3) architecture and planning (the spec, DISCUSS items, the ladder; feeds 1 and 2). Architecture and planning is anything not yet at the specificity of code. A code PR is never architecture. A rule moves out once it is enforced. Limits enforced in code belong to (1) or (2). A rule sits in (3) at rung 1-2, and moves to the division that owns the property once enforced (rung 3-4).
- `division-ownership` (architecture-and-planning, rung 1, no enforcer): Each top-level division owns its status tracking and metrics, and controls its own soundness, testing and measurability.
- `division-dogfood` (architecture-and-planning, rung 1, no enforcer): meta-dev is an ordinary project with the same three divisions (principle 4). The steward and triage sit in its architecture and planning division.
- `division-promotion` (architecture-and-planning, rung 1, no enforcer): Promotion: a concern that reaches ladder rung 2 (a precise property) may become a division; architecture and planning does the typing.
- `domain-declared` (architecture-and-planning, rung 1, no enforcer): Only what is measured can be budgeted, so every division declares its domain as typed scopes in policy: crates, paths, CI workflows, tests, plus its implicit items (log entries labelled with it).
- `domain-subset` (architecture-and-planning, rung 1, no enforcer): A child's domain is a strict subset of its parent's.
- `domain-defaults` (architecture-and-planning, rung 1, no enforcer): Defaults: all code belongs to features-and-maintenance and, through CI performance (which is always measured, unlike app performance), to performance-and-cost. Architecture-and-planning has no code.
- `domain-budget-measure` (architecture-and-planning, rung 1, no enforcer): A budget is a measure over its division's domain (test or workflow wall time, lint findings over crates or paths, open Criticals over items). A budget whose measure needs a scope the division doesn't declare is refused when the policy is read.
- `domain-reach` (architecture-and-planning, rung 1, no enforcer): A PR is within a budget's reach when its diff touches the domain or its linked task is in the division.
- `domain-scope-kinds` (architecture-and-planning, rung 1, no enforcer): Which scope kinds a budget needs depends on the budget; wall-time budgets on tests and workflows are the easiest first case.
- `concern-definition` (architecture-and-planning, rung 1, no enforcer): A concern is anything noticed that might matter, vague or precise, and is the only entity that carries a ladder rung.
- `concern-rungs` (architecture-and-planning, rung 1, no enforcer): Rungs: 1 identified (text); 2 defined (points at a spec rule); 3 checked or budgeted (a named enforcer: lint, test or CI step); 4 impossible by construction (a type).
- `concern-moves` (architecture-and-planning, rung 1, no enforcer): Moves up and down are events with evidence; a concern never claims a rung its evidence no longer supports.
- `concern-division` (architecture-and-planning, rung 1, no enforcer): A concern's division is architecture-and-planning at rungs 1-2, then the owning division at 3-4.
- `concern-resting` (architecture-and-planning, rung 1, no enforcer): Concerns may rest at any rung, but resting is budgeted with a ratchet (exact shape being decided).
- `concern-rung-only-axis` (architecture-and-planning, rung 1, no enforcer): Rung is the only typed axis; confidence is not separate, because climbing from rung 1 to 2 already narrows it.
- `concern-closing` (architecture-and-planning, rung 1, no enforcer): Closing: resolved or retired.
- `rule-decomposition` (architecture-and-planning, rung 1, no enforcer): 'Spec rule' is decomposed into four things: claim, definition, plan and decision. PR #28 (the rung audit of the old rules) is paused as input to the split.

## Severity per division (owner, 2026-10-26)
- `severity-scale` (architecture-and-planning, rung 1, no enforcer): Each division owns its severity scale. Deeper divisions have finer scales.
- `severity-mapping` (architecture-and-planning, rung 1, no enforcer): Each child level maps to exactly one parent level, and the order is kept (a more severe child level never maps below a less severe one). Parent caps count every child item that maps to the level.
- `severity-caps` (architecture-and-planning, rung 1, no enforcer): Caps are standing budgets per level, split down the tree. Rating into a full level needs a downgrade elsewhere or owner sign-off.
- `severity-evidence` (architecture-and-planning, rung 1, no enforcer): A higher level needs stronger evidence (e.g. Critical: a reproduction; Major: a concrete failure scenario).
- `severity-raising` (architecture-and-planning, rung 1, no enforcer): The reporter proposes a level. Raising after the rating is for the owner or the reviewer who reproduced it; lowering is free.
- `severity-gates-work` (architecture-and-planning, rung 1, no enforcer): Severity orders the steward's work and gates new work (an open Critical stops new work in its division).

## Build order (2026-10-26)
- `build-1-types` (architecture-and-planning, rung 1, no enforcer): Step 1: Division and Budget types; policy entries with ids.
- `build-2-log-divisions` (architecture-and-planning, rung 1, no enforcer): Step 2: every log item carries a division; claims become capacity assignments; typed Discussion status; TaskLinked.
- `build-3-status` (architecture-and-planning, rung 1, no enforcer): Step 3: `meta status` with the local record; `meta claim` worktrees and identity.
- `build-4-heartbeat` (architecture-and-planning, rung 1, no enforcer): Step 4: heartbeat timer and `meta gc`.
- `build-5-budget-domains-checker` (architecture-and-planning, rung 1, no enforcer): Step 5: budget core, first part: division domains plus one checker.
- `build-6-budget-test-workflow-time` (architecture-and-planning, rung 1, no enforcer): Step 6: budget core, then per-test and per-workflow wall time.
- `build-7-budget-standing` (architecture-and-planning, rung 1, no enforcer): Step 7: budget core, then standing budgets from the log.
- `build-8-budget-warnings` (architecture-and-planning, rung 1, no enforcer): Step 8: budget core, then warnings in status.
- `build-9-budget-task-effort` (architecture-and-planning, rung 1, no enforcer): Step 9: budget core, then per-task effort.
- `build-10-machine-and-pool` (architecture-and-planning, rung 1, no enforcer): Step 10: machine-layer leases and the agent pool.
- `build-11-severity` (architecture-and-planning, rung 1, no enforcer): Step 11: severity scales, mapping and caps.
- `build-12-waivers` (architecture-and-planning, rung 1, no enforcer): Step 12: waivers replace baselines (this also settles first-baseline seeding).
- `build-13-rungs` (architecture-and-planning, rung 1, no enforcer): Step 13: ladder rungs on concerns.

## Agent wall time (owner, 2026-10-26)
- `wall-time-guard` (architecture-and-planning, rung 1, no enforcer): Wall time is the only reliable guard against silent agent stalls. Activity signals (pushes, log events, idleness) are not: an agent blocked on a watcher that can never fire looks idle while working and busy while stuck.
- `wall-time-budget` (architecture-and-planning, rung 1, no enforcer): Every claimed task carries a wall-time budget, set when it is claimed (a project default in tooling/policy.toml, overridable per task).
- `wall-time-spent` (architecture-and-planning, rung 1, no enforcer): Spending it is an event, whatever the agent appears to be doing. The task is flagged and escalated: the lead checks the agent and replaces it, or extends the budget explicitly (a recorded, budgeted extension).
- `wall-time-release` (architecture-and-planning, rung 1, no enforcer): A task whose budget expires again without an extension is released, so another agent can claim it.
- `wall-time-headroom` (architecture-and-planning, rung 1, no enforcer): Headroom applies: a task past ~60-80% of its budget is a warning to the steward.
- `wall-time-example` (architecture-and-planning, rung 1, no enforcer): Example: a mutation-testing PR's agent waited on a merge in a repo that had been frozen; nothing fired until the owner noticed by hand.

## Status, part 1: local state (owner, 2026-10-26)
- `status-local-never-remote` (architecture-and-planning, rung 1, no enforcer): Local file-tree facts and settings never go to the remote.
- `status-local-settings` (architecture-and-planning, rung 1, no enforcer): Settings (worktree root, target-dir layout, gc retention) live in a gitignored local config per clone, such as meta.local.toml (ignore rule committed, defaults when absent).
- `status-local-record` (architecture-and-planning, rung 1, no enforcer): The record of worktrees and target dirs (paths, creation, local trail) is typed, written only by meta, and kept inside the clone's git dir, never pushed.
- `status-local-claim-worktree` (architecture-and-planning, rung 1, no enforcer): `meta claim` creates the task's worktree under the configured root and records it. Agents make no ad-hoc clones.
- `status-local-join` (architecture-and-planning, rung 1, no enforcer): Status joins the record to the log by task id.
- `status-local-lost-record` (architecture-and-planning, rung 1, no enforcer): If the record is lost, gc deletes nothing and reports leftovers.

## Status, part 2: external state (owner, 2026-10-26)
- `status-external-live` (architecture-and-planning, rung 1, no enforcer): PR and CI state is read live from GitHub on each run, with one batched query per project, and is never copied into the log (GitHub is its single home).
- `status-external-history` (architecture-and-planning, rung 1, no enforcer): History such as time red or review rounds is also derived from GitHub.
- `status-external-unknown` (architecture-and-planning, rung 1, no enforcer): If GitHub can't be reached, the state shows as unknown, and the heartbeat treats unknown as needing attention.

## Status, part 3: task-to-PR links (owner, 2026-10-26)
- `status-links-branch` (architecture-and-planning, rung 1, no enforcer): `meta claim` names the task's branch.
- `status-links-event` (architecture-and-planning, rung 1, no enforcer): When a PR is opened from a claimed task's branch, the hook writes a TaskLinked { task, pr } event to the log.
- `status-links-several` (architecture-and-planning, rung 1, no enforcer): A task can link several PRs, as in a gh stack.
- `status-links-orphan-pr` (architecture-and-planning, rung 1, no enforcer): Status flags any open PR that has no task.
- `status-links-pr-state-live` (architecture-and-planning, rung 1, no enforcer): PR state itself is still read live from GitHub (part 2).

## Status, part 4: agents, metrics and worktree lifetime (owner, 2026-10-26)
- `status-agents-independent` (architecture-and-planning, rung 1, no enforcer): Agents are an implementation detail, independent of tasks (rotating agents or dedicated pools, never 1:1); a claim becomes an assignment of agent capacity to a task.
- `status-metrics-per-agent` (architecture-and-planning, rung 1, no enforcer): Metrics per agent, attributed to the task it is working on: (1) tokens consumed; (2) wall time; (3) compute (CPU and memory).
- `status-metrics-wall-time` (architecture-and-planning, rung 1, no enforcer): Wall time diverges from tokens (for example a stalled script) and works at two scales: small (is this agent stalled right now?) and large (is the task on schedule against its due time?).
- `status-metrics-compute-pool` (architecture-and-planning, rung 1, no enforcer): Compute (CPU and memory) is drawn from a central machine budget just as agents are drawn from a central pool.
- `status-task-budgets` (architecture-and-planning, rung 1, no enforcer): Task budgets: effort (tokens plus compute) and a due time set by priority.
- `status-stalled` (architecture-and-planning, rung 1, no enforcer): Stalled means a small-scale stall, past due with no assigned capacity, or effort spent with no recorded extension.
- `status-worktree-lifetime` (architecture-and-planning, rung 1, no enforcer): A worktree belongs to a PR's branch; gc deletes it when that PR is merged, closed or stalled (resuming is cheap: recreate it from the remote), after the final check that all work is on the remote and nothing has it open.

## Status, part 5: discussion status, tokens, compute (owner, 2026-10-26)
- `status-discussion-typed` (architecture-and-planning, rung 1, no enforcer): Discussion is its own fully typed task status, not a free-form blocker: a task waiting on an owner discussion is in Discussion(issue id) and is never counted as stalled.
- `status-waits-typed` (architecture-and-planning, rung 1, no enforcer): Other waits are typed too (on another task, on something external).
- `status-tokens-transcripts` (architecture-and-planning, rung 1, no enforcer): Tokens are measured from Claude Code's local transcripts: each assistant message carries a usage record (input, cache, output tokens), and each subagent has its own transcript; hooks get transcript_path.
- `status-tokens-storage` (architecture-and-planning, rung 1, no enforcer): Per-task totals go to the log; raw transcripts stay local.
- `status-compute-lease` (architecture-and-planning, rung 1, no enforcer): Compute is budgeted, not recorded: a task must hold a CPU and memory lease from a local, per-machine pool before running heavy commands (builds, test suites, mutants, Chrome checks); the pool is sized to the machine, and anything without a lease waits.

## Status, part 6: heartbeat and output (owner, 2026-10-26)
- `status-heartbeat-job` (architecture-and-planning, rung 1, no enforcer): A launchd job runs `meta status` and `meta gc` every few minutes, writing flags to the local record at no token cost.
- `status-heartbeat-wake` (architecture-and-planning, rung 1, no enforcer): Flags needing judgement (stalled, red PR with no capacity, past due, stuck command) wake a lead Claude session, which acts and goes back to sleep; for now that is the lead's scheduled wakeups.
- `status-output` (architecture-and-planning, rung 1, no enforcer): Output: a terminal tree of divisions down to tasks (state, PR and CI, assigned capacity, effort against due time), with flags on top; a live dashboard page later if needed.

## Machine layer (owner, 2026-10-26)
- `machine-ownership` (architecture-and-planning, rung 1, no enforcer): The machine layer is per machine and owned by no project.
- `machine-config-state` (architecture-and-planning, rung 1, no enforcer): Config is in ~/.config/meta/machine.toml (CPU slots, memory, disk floor, maximum concurrent agents) and state is in ~/.local/state/meta.
- `machine-lease` (architecture-and-planning, rung 1, no enforcer): Heavy commands run under meta lease (file-lock slots that a crash releases automatically, sizing CARGO_BUILD_JOBS).
- `machine-hook-refusal` (architecture-and-planning, rung 1, no enforcer): The hook refuses unwrapped heavy commands.
- `machine-queue` (architecture-and-planning, rung 1, no enforcer): Waiting commands are served by priority.
- `machine-timed-runs` (architecture-and-planning, rung 1, no enforcer): Timed runs take a full-machine lease.
- `machine-visibility` (architecture-and-planning, rung 1, no enforcer): Projects see only grant or wait; the owner sees everything.
- `machine-disk-floor` (architecture-and-planning, rung 1, no enforcer): Disk floor: `meta claim` refuses new worktrees below it.
- `machine-order` (architecture-and-planning, rung 1, no enforcer): The machine layer comes after `meta status`.

## Token measurement (owner, 2026-10-26)
- `tokens-measure-only` (architecture-and-planning, rung 1, no enforcer): For now, only measure tokens per model (input, cache write, cache read, output) from local transcripts, per session and subagent, attributed to tasks.
- `tokens-budgeting-deferred` (architecture-and-planning, rung 1, no enforcer): Budgeting is DEFERRED: the unit in multiples of the Claude Pro plan's limit, calibration from limit hits (first anchor 2026-10-26 about 16:10Z), and headroom.

## Commit identity guard (owner, 2026-10-26)
- `identity-noreply-only` (architecture-and-planning, rung 1, no enforcer): No commit may carry an email that isn't a GitHub noreply address.
- `identity-hook` (architecture-and-planning, rung 1, no enforcer): Enforced in three places, (1): the PreToolUse hook refuses git commit, and rebase or amend that rewrite commits, when the effective identity isn't noreply, naming the git config commands that fix it.
- `identity-ci` (architecture-and-planning, rung 1, no enforcer): Enforced in three places, (2): cargo xtask ci (lint) fails on any commit in the PR range whose author or committer email isn't noreply.
- `identity-claim` (architecture-and-planning, rung 1, no enforcer): Enforced in three places, (3): `meta claim` sets the noreply identity on worktrees it creates.
- `identity-address-source` (architecture-and-planning, rung 1, no enforcer): The noreply address comes from local config, not from the repo.

## Agent pool and model tiers (owner, 2026-10-26)
- `pool-shared` (architecture-and-planning, rung 1, no enforcer): Agents are drawn from one shared pool, allocated centrally.
- `pool-no-nested-agents` (architecture-and-planning, rung 1, no enforcer): Implementer agents may not start subagents on their own: on 2026-10-26 nested agents and forks multiplied model usage unseen and hit the weekly limit.
- `pool-allocator-spend` (architecture-and-planning, rung 1, no enforcer): Spawning should go through the allocator, and the spend (agents, tokens, wall time) should be recorded against the performance-and-cost division's budget.
- `pool-spawn-hook` (architecture-and-planning, rung 1, no enforcer): A PreToolUse hook on the Agent tool should refuse to spawn without a grant.
- `tier-difficulty-rating` (architecture-and-planning, rung 1, no enforcer): Every task is rated by difficulty, and very mechanical tasks go to agents on smaller models.
- `tier-scale` (architecture-and-planning, rung 1, no enforcer): Proposed scale: Mechanical (rename, apply a known pattern, call-site updates, named consts) to Haiku; Routine (a bounded implementation against a clear spec) to Sonnet; Hard (design, adversarial review, tricky correctness such as #7 and #8) to Opus.
- `tier-typed-field` (architecture-and-planning, rung 1, no enforcer): The rating is a typed field on the task that the agent-pool allocator reads.
- `tier-escalation` (architecture-and-planning, rung 1, no enforcer): When a cheap tier fails review or CI, the task escalates one tier and the miss is recorded, so ratings calibrate over time.
- `tier-spend` (architecture-and-planning, rung 1, no enforcer): Spend is counted against the performance-and-cost budget.

# Not yet converted: hand-written prose, 1846 words

Source: Basho postmortem, part 3 (kept in the Basho project's private repo).

## Core principles (owner)
1. Everything is tracked and leaves a trail.
2. Everything has a budget that is spent against.
3. Everything has a single source of truth.
4. Everything dogfoods itself: the principles and the enforcement ladder apply to the meta-repo, its rules, its budgets and its tracker too.
5. Projects are isolated: each defines its own development config, and no project needs to know anything about any other project.

## Entities (first cut, to be derived from the principles)
- Feature
- Claim
- Evidence
- Artifact
- Task
- Issue
- Procedure/Tool (with maturity level)
- Run
- Budget
- Waiver
- Resource/Lease
- Actor

## Open questions
- Where truth lives: slow-changing typed truth in git vs fast-changing state (claims, leases, runs) in a service; one schema for both?
- Tracker location/views (GitHub Issues / Linear / own): downstream of the above.

## Severity is budgeted (owner)
- Budgeting is the core enforcement method; it stops severity being gamed by declaring everything "critical".
- Mechanisms:
  - Explicit cap: a fixed allowance per severity level (count or share). Raising one item means lowering another or spending budget.
  - Implicit cost: higher severity demands stronger evidence, which is paid from the verification/CI budget. Over-rating gets expensive.
  - Calibration: after real use, compare ratings with what actually broke. Misses and false alarms adjust future allowances.
- Severity definitions still apply (critical = crash, blocking UI/navigation, or wrong plan information); the budget limits how many items may claim each level.
- Implication: enforcement is mostly principle 2 (budgets). Typing is the means of principle 3.

## Enforcement ladder (owner): every concern climbs it
1. Identified and tracked: even something vague, even a feeling that something is wrong.
2. Defined specifically, unambiguously, clearly and comprehensively, i.e. typestated. If it can't be expressed in the type system, it isn't specific or unambiguous enough yet.
3. Enforced by a budget.
4. Made impossible/unrepresentable by construction.
- Applies to everything, including meta-level concerns (principle 4).

## Single source of truth (principle 3), decided 2026-10-22
- Each fact has exactly one home; everywhere else is a derived, read-only projection.
- Definitions (features, claim definitions, rules, budgets, severity allowances, procedures, schema): typed Rust in git, compiled, reviewed, owner-approved changes.
- Events (claims of tasks, runs, evidence, leases, waivers, issues, budget spend): an append-only typed event log, validated by the same Rust types. Current state = a fold over the log.
- External systems keep their own facts (GitHub owns PR state); the log mirrors them as events.
- Storage of the event log: START IN GIT (per-actor append-only files; leases via atomic ref updates). No server yet.
  - Dogfooded: the git backend's own costs are a tracked, budgeted meta-item:
    - append latency;
    - write conflicts/retries;
    - lease contention and wait time;
    - GitHub rate-limit hits;
    - query time over the log.
  - Overspending that budget is the trigger to move to a hosted service (live store + periodic git snapshots).
  - The storage backend sits behind one interface (a trait), so the move doesn't change any caller.

## Layout (decided 2026-10-22)
- Repo name: **meta-development**.
- Crates:
  - meta-schema (types only)
  - meta-log (trait + git backend)
  - meta-state (folds)
  - meta-cli (`meta` command)
  - meta-ci (budgets, timing, flaky, affected tests, evidence check)
  - meta-lints (code + process lints)
  - meta-procedure (trail-recording runner)
- Also:
  - procedures/
  - plugin/ (Claude Code: roles, skills, hooks)
  - templates/
  - self/ (dogfood: the repo's own spec, claims, budgets)
- Consumer project:
  - xtask depends on meta-ci/lints/procedure (pinned);
  - a `spec` crate in meta-schema types;
  - the plugin installed.
- Event log: ONE PER PROJECT (owner). This is also the most straightforward way for meta-development to dogfood itself (it has its own log).
  - DECIDED: a separate never-merged branch in the project repo (e.g. `meta/log`).
    - Written directly by meta-cli regardless of the agent's working branch.
    - Atomic push = lock (CAS).
    - Main stays clean; no CI trigger.
    - Fallback if separate permissions are ever needed: a separate log repo per project.
- Shared limits: NO cross-project view (principle 5). Each project sets its own budgets in its own config. Isolate physical resources by construction where possible (e.g. each project creates its own simulators).

## Schema notes (2026-10-22)
- First sketch of types accepted in direction (see conversation): typed Ids, Feature/Claim/Severity with CriticalReason, Budget/Measure, ladder typestate Concern<L>, Evidence enum, Event enum.
- OWNER: TRAITS are first-class. They unify disparate types and define affordances, e.g. "knowing when something is" is a trait.
- Confidence: a PROCEDURE for now (different formulas for different situations). Its audit trail gradually informs turning it into code (tooling ladder).
- Claims (typed, 2026-10-22): Claim = Property<S: Subject> + Scope<S> (One | Where(filter) | Every) + severity. Each property is its own type, with Checkability: Executable(fn(&S)->Verdict) or External.
  - Evidence covers a property over a scope (type = Every by construction; test/check = instances examined; source = One, as of date).
- META: a "vague string" budget (owner). Untyped meaning-bearing text (Text statements, notes standing in for types) is counted and budgeted, ratcheting down, to force progressive tightening into concrete, explicit definitions.
- NEXT: define Subjects, unified with traits (code vs data vs world facts...).
- Subject kinds (draft): Code, Data (our records), World (what data describes, goes stale), Process (procedures, checks, agent roles), Meta.
  - Owner: "behaviour" splits into FEATURES and INVARIANTS.
- Subject traits (draft): Subject: Identified + describe; Located; Versioned; Dated; Represents{World}; Runnable; Owned.
  - Executable properties need Runnable/data access; External need Represents/Dated.
  - Freshness for confidence: Versioned (stale on version change) / Dated (stale with time).
- Features = Subjects: severity, owner; status COMPUTED from evidence on acceptance claims (Implemented, Accepted(scenario), Reachable...).
- Invariants = claims with Scope::AllStates(StateSpace<S>); tied to property-based testing (proptest; state-machine testing for stateful subjects, with a reference model).
  - Evidence::PropertyTest { invariant, cases, seed, regressions (shrunk counterexamples kept as artifacts and rerun) }.
  - Strength: construction > property tests (cases, generator coverage) > hand-picked scenarios.
  - Example (Basho): "the tracker never moves before a step the user set". NOT "never moves backward": it must move back after a mistaken forward jump.
- OWNER: Basho is a small subset of possible projects. Design for the GENERAL CASE first; Basho examples are illustrations only.
- General-case additions (from checking library / service / pipeline / ML / infra / docs projects):
  1. Environment as a subject kind or claim context: claims that hold "in production" or "on this device".
  2. Dependency as a subject kind: licences, vulnerabilities, versions.
  3. Measured/statistical evidence: a value + uncertainty + sample size; properties compare it to thresholds. This also covers perf profiles and budget measurements.
  4. Contracts across versions: properties relating two versions of a subject (e.g. no public API break).
  - Also: datasets as versioned subjects with distribution-level claims; users' observations as Person evidence; documents as subjects with citations as Source evidence.

## Next: plugin and hooks
- Plugin (agreed 2026-10-25):
  - Roles:
    - lead: priorities and tasks; no merging; no rule edits without owner;
    - worker: one task → one PR; can't close without evidence;
    - verifier: unprimed, goal + constraints only;
    - researcher: procedures; no contact, no forms, fixed UA;
    - tester: only via the trail wrapper.
  - Day-one hooks:
    - block merge, push to main, force-push, protected-path reads, unasked installs, self-killing pkill;
    - auto-push after commit;
    - egress policy (project UA, no email, no configured personal data);
    - procedure-owned commands only via `meta run`;
    - auto-timing of long commands as budget measurements;
    - Stop blocked until the task has a status event;
    - SessionStart injects state from the log (replaces handoff notes).
  - meta-cli exposed as an MCP server; procedures as skills.
  - Hooks are ladder level 3; server-side (branch protection, required checks, log refusing overspend) is the backstop.
- All project-specific values come from project config (principle 5).
- COMPLEXITY BUDGET (owner): the complement to the vagueness/coverage budget, against the opposite vice of bikeshedding / over-formalising.
  - Spent by: new types, traits, enum variants, event kinds, rules, procedures, hooks, schema surface; lines of meta code.
  - A new formalism must justify itself: name the concern(s) it moves up the ladder, or the vagueness it removes.
  - Unused or redundant structure is paid down (deleted).
  - Tension by design: the vagueness budget pushes to define; the complexity budget pushes to define only what earns its keep.

## Goal: sight-unseen merges (owner, 2026-10-25)
- Today the owner reviews and merges every PR. That is a bottleneck, worse as implementer agents parallelise.
- GOAL: the system becomes robust enough that most PRs can be merged sight-unseen.
- Treated like everything else (ladder + budgets):
  - Merge authority is earned per risk tier, computed from what the PR touches (severity of affected features/claims; rule or budget changes; schema changes).
  - A PR auto-merges only if the evidence check passes and its tier is within the currently granted authority.
  - Authority is widened by track record:
    - post-merge defect rate;
    - revert rate;
    - audit disagreement on sampled auto-merged PRs.
    Authority is narrowed automatically when these overspend their budgets.
  - Always owner-reviewed: changes to rules, budgets, enforcement config and authority itself (CODEOWNERS).
- Until then the owner-merges rule stays.

## Goal: automate the owner's triage/priority/wall-time role (owner, 2026-10-25)
- Today the owner does it: flagging slow CI, noticing a PR like Basho #72 blocking everything, spotting perf problems in the app AND in devx (builds, tests, simulators, agent waits).
- GOAL: the repo automates most of this. A STEWARD role, run on a schedule and on events, reads the log and:
  - flags budget overruns and trends before they breach: CI/check wall time, build time, queue/lease wait, agent wall time, flakiness;
  - detects blocking work:
    - PR age;
    - review rounds;
    - number of tasks/PRs waiting on it;
    - critical-path length;
    - a PR growing in scope (#72's pattern);
  - tracks devx perf like app perf: profiles, history, headroom (optimisation ledger);
  - proposes priorities from severity × evidence gap × what is blocked;
  - files concerns/tasks itself (with trails).
- The steward escalates to the owner only for decisions: budget or authority changes, priority conflicts, waivers beyond budget.
- Dogfooded: the steward's own false alarms and misses are budgeted, and calibrated against what the owner would have flagged.
- Depends on stage 1 (runs, timings, budgets). Its first version belongs there.

## Prose (owner, 2026-10-25)
Prose is allowed only in three categories, each clearly marked and separated from the rest of the codebase:
1. Agent context: occasionally relevant to agents. May bloat somewhat; consolidated aggressively from time to time; not meant for humans.
2. Documentation: legible to humans and agents; the bare minimum; concrete, with examples; always STE100.
3. Strings attached to structured data: terse (e.g. audit-log entries).
Every other surface contains no prose. Enforced by lints (meta-lints); counted against the vagueness budget.
- Classification and locations: CLAUDE.md.
- Stage 4 moves Basho's lints (incl. no_comments) into meta-lints. meta-development should adopt them as early as possible (dogfooding).
