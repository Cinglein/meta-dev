# Meta-development repo: spec (draft, started 2026-10-22)

Source: Basho postmortem (the Basho project's private repo, part 3).

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

## Next: bootstrapping (smallest useful first version, build order)

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
  - flags budget overruns and trends before they breach: CI/check wall time, build time, queue/lease wait, agent idle time, flakiness;
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
- Classification (owner):
  - Code comments, including `///` docs: BANNED outright (as in the chess lints; Basho has xtask/src/task/no_comments.rs). Meaning lives in names and types.
  - Error and help text: category 3.
  - Commit messages, PR titles and descriptions: category 3 (loosely structured data). Keep them terse.
  - SPEC.md and the postmortem: agent context (1).
  - A short STE100 README and usage guide: documentation (2).
- Stage 4 moves Basho's lints (incl. no_comments) into meta-lints. meta-development should adopt them as early as possible (dogfooding).
