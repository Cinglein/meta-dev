# Meta-development repo: spec (draft, started 2026-10-22)

Sections down to the next top-level heading are rendered from the rule records in `self/spec`; edit those, then run `cargo run -p meta-spec`. Each rule shows its id, division, ladder rung and enforcer.

## Divisions (owner, 2026-10-26)
- `division-is-a-property` (architecture-and-planning, rung 1, no enforcer): A division is any behaviour or invariant made first class (e.g. Basho navigation). It is a named property, not a code area: divisions overlap freely in code and are isolated by tracking (own items, budgets, checks, status).
- `division-context` (architecture-and-planning, rung 1, no enforcer): An agent on a division sees only that division's context.
- `division-nesting` (architecture-and-planning, rung 1, no enforcer): Divisions nest arbitrarily. A child gets a subset of its parent's budget.
- `division-membership` (architecture-and-planning, rung 1, no enforcer): Every concern, issue and task belongs to exactly one division. An unplaced item belongs to the project root; growth there signals a missing division.
- `division-top-level` (architecture-and-planning, rung 1, no enforcer): Top level, in every project: (1) features and maintenance (navigation, the tracker, guarding lints and mutants); (2) performance and cost (CI and test time, mutation time, Actions minutes and money, disk, agent wall time); (3) architecture and planning (the spec, DISCUSS items, the ladder; feeds 1 and 2).
- `division-ownership` (architecture-and-planning, rung 1, no enforcer): Each top-level division owns its status tracking and metrics, and controls its own soundness, testing and measurability.
- `division-dogfood` (architecture-and-planning, rung 1, no enforcer): meta-dev is an ordinary project with the same three divisions (principle 4). The steward and triage sit in its architecture and planning division.
- `division-promotion` (architecture-and-planning, rung 1, no enforcer): Promotion: a concern that reaches ladder rung 2 (a precise property) may become a division; architecture and planning does the typing.

## Budgets (owner, 2026-10-26)
- `budget-definition` (architecture-and-planning, rung 1, no enforcer): A budget is a typed definition with an id, in tooling/policy.toml. Spend events in the log point at it.
- `budget-unit` (architecture-and-planning, rung 1, no enforcer): Unit, exactly one per budget: Count, Duration, Share (later maybe Money or CI minutes).
- `budget-window` (architecture-and-planning, rung 1, no enforcer): Window: per change (ratchet against the merge base), rolling per period, or standing (open items at any time).
- `budget-overspend` (architecture-and-planning, rung 1, no enforcer): Overspend action is part of the definition: block (default) or escalate to DISCUSS.
- `budget-measured-spend` (architecture-and-planning, rung 1, no enforcer): Spend is recorded by measurement only (CI and tools). Severity is the judged exception, checked later by calibration.
- `budget-raise` (architecture-and-planning, rung 1, no enforcer): Raising a budget is owner-only.
- `budget-waivers` (architecture-and-planning, rung 1, no enforcer): Waivers are spend from the waiver budget of the division whose rule they bend. They replace raw lint baselines.

## Severity per division (owner, 2026-10-26)
- `severity-scale` (architecture-and-planning, rung 1, no enforcer): Each division owns its severity scale. Deeper divisions have finer scales.
- `severity-mapping` (architecture-and-planning, rung 1, no enforcer): Each child level maps to exactly one parent level, and the order is kept (a more severe child level never maps below a less severe one). Parent caps count every child item that maps to the level.
- `severity-caps` (architecture-and-planning, rung 1, no enforcer): Caps are standing budgets per level, split down the tree. Rating into a full level needs a downgrade elsewhere or owner sign-off.
- `severity-evidence` (architecture-and-planning, rung 1, no enforcer): A higher level needs stronger evidence (e.g. Critical: a reproduction; Major: a concrete failure scenario).
- `severity-raising` (architecture-and-planning, rung 1, no enforcer): The reporter proposes a level. Raising after the rating is for the owner or the reviewer who reproduced it; lowering is free.
- `severity-gates-work` (architecture-and-planning, rung 1, no enforcer): Severity orders the steward's work and gates new work (an open Critical stops new work in its division).

## Build order (2026-10-26)
- `build-1-types` (architecture-and-planning, rung 1, no enforcer): Step 1: Division and Budget types; policy entries with ids.
- `build-2-log-divisions` (architecture-and-planning, rung 1, no enforcer): Step 2: every log item carries a division.
- `build-3-status` (architecture-and-planning, rung 1, no enforcer): Step 3: `meta status` as a tree of divisions (tasks, claims, PRs and CI, agents with worktree and last activity).
- `build-4-steward` (architecture-and-planning, rung 1, no enforcer): Step 4: steward heartbeat and `meta gc` on top of status.
- `build-5-severity` (architecture-and-planning, rung 1, no enforcer): Step 5: severity scales, mapping and caps.
- `build-6-waivers` (architecture-and-planning, rung 1, no enforcer): Step 6: waivers replace baselines (this also settles first-baseline seeding).
- `build-7-rungs` (architecture-and-planning, rung 1, no enforcer): Step 7: ladder rungs on concerns.

# Not yet converted: hand-written prose, 2006 words

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

## Agent wall time (owner, 2026-10-26)
- Wall time is the only reliable guard against silent agent stalls. Activity signals (pushes, log events, idleness) are not: an agent blocked on a watcher that can never fire looks idle while working and busy while stuck.
- Every claimed task carries a wall-time budget, set when it is claimed (a project default in tooling/policy.toml, overridable per task).
- Spending it is an event, whatever the agent appears to be doing. The task is flagged and escalated: the lead checks the agent and replaces it, or extends the budget explicitly (a recorded, budgeted extension).
- A task whose budget expires again without an extension is released, so another agent can claim it.
- Headroom applies: a task past ~60-80% of its budget is a warning to the steward.
- Example: a mutation-testing PR's agent waited on a merge in a repo that had been frozen; nothing fired until the owner noticed by hand.

## Prose (owner, 2026-10-25)
Prose is allowed only in three categories, each clearly marked and separated from the rest of the codebase:
1. Agent context: occasionally relevant to agents. May bloat somewhat; consolidated aggressively from time to time; not meant for humans.
2. Documentation: legible to humans and agents; the bare minimum; concrete, with examples; always STE100.
3. Strings attached to structured data: terse (e.g. audit-log entries).
Every other surface contains no prose. Enforced by lints (meta-lints); counted against the vagueness budget.
- Classification and locations: CLAUDE.md.
- Stage 4 moves Basho's lints (incl. no_comments) into meta-lints. meta-development should adopt them as early as possible (dogfooding).
