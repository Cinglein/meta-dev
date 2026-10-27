use crate::enforcement::Enforcement::Tracked;
use crate::rule::{Rule, RuleId};
use crate::section::Section;
use crate::wording::Wording;
use task::TopDivision::ArchitectureAndPlanning;

pub const SECTIONS: &[Section] = &[
    Section::new(
        Wording::new("Budgets (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("budget-definition"),
                ArchitectureAndPlanning,
                Wording::new(
                    "A budget is a typed definition with an id, in tooling/policy.toml. Spend events in the log point at it.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-unit"),
                ArchitectureAndPlanning,
                Wording::new("Unit, exactly one per budget: Count, Duration, Share (later maybe Money or CI minutes)."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-window"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Window: per change (ratchet against the merge base), rolling per period, or standing (open items at any time).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-overspend"),
                ArchitectureAndPlanning,
                Wording::new("Overspend action is part of the definition: block (default) or escalate to DISCUSS."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-measured-spend"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Spend is recorded by measurement only (CI and tools). Severity is the judged exception, checked later by calibration.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-raise"),
                ArchitectureAndPlanning,
                Wording::new("Raising a budget is owner-only."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-waivers"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Waivers are spend from the waiver budget of the division whose rule they bend. They replace raw lint baselines.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-enforce-ci"),
                ArchitectureAndPlanning,
                Wording::new("Budgets are enforced in CI (the local gate where Actions can't run)."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-enforce-per-change"),
                ArchitectureAndPlanning,
                Wording::new("Per-change budgets: measured fresh by CI."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-enforce-standing"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Standing budgets are derived from the log (open Criticals, waivers in force, concern backlog): CI folds meta/log read-only on every run. An over-budget standing budget blocks only PRs in its division, but a division whose property all code affects (for example CI performance) blocks every PR that could affect it.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-enforce-per-task"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Per-task budgets (effort, due): CI finds the PR's task via TaskLinked or the branch, and fails on overspend with no recorded extension; a PR with no task fails.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-enforce-spend-source"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Spend comes from local measurement (heartbeat from transcripts) written to the log; CI trusts it under the honest-mistakes threat model.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("budget-enforce-machine-timing"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Machine budgets (CPU, memory, agents, disk) are runtime controls enforced by hooks and leases, not CI. Timing budgets are measured only on CI or in the local gate's clean run.",
                ),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Divisions (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("division-is-a-property"),
                ArchitectureAndPlanning,
                Wording::new(
                    "A division is any behaviour or invariant made first class (e.g. Basho navigation). It is a named property, not a code area: divisions overlap freely in code and are isolated by tracking (own items, budgets, checks, status).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("division-context"),
                ArchitectureAndPlanning,
                Wording::new("An agent on a division sees only that division's context."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("division-nesting"),
                ArchitectureAndPlanning,
                Wording::new("Divisions nest arbitrarily. A child gets a subset of its parent's budget."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("division-membership"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Every concern, issue and task belongs to exactly one division. An unplaced item belongs to the project root; growth there signals a missing division.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("division-top-level"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Top level, in every project: (1) features and maintenance (navigation, the tracker, guarding lints and mutants); (2) performance and cost (CI and test time, mutation time, Actions minutes and money, disk, agent wall time); (3) architecture and planning (the spec, DISCUSS items, the ladder; feeds 1 and 2). Architecture and planning is anything not yet at the specificity of code. A code PR is never architecture. A rule moves out once it is enforced. Limits enforced in code belong to (1) or (2). A rule sits in (3) at rung 1-2, and moves to the division that owns the property once enforced (rung 3-4).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("division-ownership"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Each top-level division owns its status tracking and metrics, and controls its own soundness, testing and measurability.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("division-dogfood"),
                ArchitectureAndPlanning,
                Wording::new(
                    "meta-dev is an ordinary project with the same three divisions (principle 4). The steward and triage sit in its architecture and planning division.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("division-promotion"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Promotion: a concern that reaches ladder rung 2 (a precise property) may become a division; architecture and planning does the typing.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("domain-declared"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Only what is measured can be budgeted, so every division declares its domain as typed scopes in policy: crates, paths, CI workflows, tests, plus its implicit items (log entries labelled with it).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("domain-subset"),
                ArchitectureAndPlanning,
                Wording::new("A child's domain is a strict subset of its parent's."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("domain-defaults"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Defaults: all code belongs to features-and-maintenance and, through CI performance (which is always measured, unlike app performance), to performance-and-cost. Architecture-and-planning has no code.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("domain-budget-measure"),
                ArchitectureAndPlanning,
                Wording::new(
                    "A budget is a measure over its division's domain (test or workflow wall time, lint findings over crates or paths, open Criticals over items). A budget whose measure needs a scope the division doesn't declare is refused when the policy is read.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("domain-reach"),
                ArchitectureAndPlanning,
                Wording::new(
                    "A PR is within a budget's reach when its diff touches the domain or its linked task is in the division.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("domain-scope-kinds"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Which scope kinds a budget needs depends on the budget; wall-time budgets on tests and workflows are the easiest first case.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("concern-definition"),
                ArchitectureAndPlanning,
                Wording::new(
                    "A concern is anything noticed that might matter, vague or precise, and is the only entity that carries a ladder rung.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("concern-rungs"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Rungs: 1 identified (text); 2 defined (points at a spec rule); 3 checked or budgeted (a named enforcer: lint, test or CI step); 4 impossible by construction (a type).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("concern-moves"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Moves up and down are events with evidence; a concern never claims a rung its evidence no longer supports.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("concern-division"),
                ArchitectureAndPlanning,
                Wording::new(
                    "A concern's division is architecture-and-planning at rungs 1-2, then the owning division at 3-4.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("concern-resting"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Concerns may rest at any rung, but resting is budgeted with a ratchet (exact shape being decided).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("concern-rung-only-axis"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Rung is the only typed axis; confidence is not separate, because climbing from rung 1 to 2 already narrows it.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("concern-closing"),
                ArchitectureAndPlanning,
                Wording::new("Closing: resolved or retired."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("rule-decomposition"),
                ArchitectureAndPlanning,
                Wording::new(
                    "'Spec rule' is decomposed into four things: claim, definition, plan and decision. PR #28 (the rung audit of the old rules) is paused as input to the split.",
                ),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Severity per division (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("severity-scale"),
                ArchitectureAndPlanning,
                Wording::new("Each division owns its severity scale. Deeper divisions have finer scales."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("severity-mapping"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Each child level maps to exactly one parent level, and the order is kept (a more severe child level never maps below a less severe one). Parent caps count every child item that maps to the level.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("severity-caps"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Caps are standing budgets per level, split down the tree. Rating into a full level needs a downgrade elsewhere or owner sign-off.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("severity-evidence"),
                ArchitectureAndPlanning,
                Wording::new(
                    "A higher level needs stronger evidence (e.g. Critical: a reproduction; Major: a concrete failure scenario).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("severity-raising"),
                ArchitectureAndPlanning,
                Wording::new(
                    "The reporter proposes a level. Raising after the rating is for the owner or the reviewer who reproduced it; lowering is free.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("severity-gates-work"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Severity orders the steward's work and gates new work (an open Critical stops new work in its division).",
                ),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Build order (2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("build-1-types"),
                ArchitectureAndPlanning,
                Wording::new("Step 1: Division and Budget types; policy entries with ids."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-2-log-divisions"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Step 2: every log item carries a division; claims become capacity assignments; typed Discussion status; TaskLinked.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-3-status"),
                ArchitectureAndPlanning,
                Wording::new("Step 3: `meta status` with the local record; `meta claim` worktrees and identity."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-4-heartbeat"),
                ArchitectureAndPlanning,
                Wording::new("Step 4: heartbeat timer and `meta gc`."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-5-budget-domains-checker"),
                ArchitectureAndPlanning,
                Wording::new("Step 5: budget core, first part: division domains plus one checker."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-6-budget-test-workflow-time"),
                ArchitectureAndPlanning,
                Wording::new("Step 6: budget core, then per-test and per-workflow wall time."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-7-budget-standing"),
                ArchitectureAndPlanning,
                Wording::new("Step 7: budget core, then standing budgets from the log."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-8-budget-warnings"),
                ArchitectureAndPlanning,
                Wording::new("Step 8: budget core, then warnings in status."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-9-budget-task-effort"),
                ArchitectureAndPlanning,
                Wording::new("Step 9: budget core, then per-task effort."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-10-machine-and-pool"),
                ArchitectureAndPlanning,
                Wording::new("Step 10: machine-layer leases and the agent pool."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-11-severity"),
                ArchitectureAndPlanning,
                Wording::new("Step 11: severity scales, mapping and caps."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-12-waivers"),
                ArchitectureAndPlanning,
                Wording::new("Step 12: waivers replace baselines (this also settles first-baseline seeding)."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-13-rungs"),
                ArchitectureAndPlanning,
                Wording::new("Step 13: ladder rungs on concerns."),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Agent wall time (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("wall-time-guard"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Wall time is the only reliable guard against silent agent stalls. Activity signals (pushes, log events, idleness) are not: an agent blocked on a watcher that can never fire looks idle while working and busy while stuck.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("wall-time-budget"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Every claimed task carries a wall-time budget, set when it is claimed (a project default in tooling/policy.toml, overridable per task).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("wall-time-spent"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Spending it is an event, whatever the agent appears to be doing. The task is flagged and escalated: the lead checks the agent and replaces it, or extends the budget explicitly (a recorded, budgeted extension).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("wall-time-release"),
                ArchitectureAndPlanning,
                Wording::new(
                    "A task whose budget expires again without an extension is released, so another agent can claim it.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("wall-time-headroom"),
                ArchitectureAndPlanning,
                Wording::new("Headroom applies: a task past ~60-80% of its budget is a warning to the steward."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("wall-time-example"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Example: a mutation-testing PR's agent waited on a merge in a repo that had been frozen; nothing fired until the owner noticed by hand.",
                ),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Status, part 1: local state (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("status-local-never-remote"),
                ArchitectureAndPlanning,
                Wording::new("Local file-tree facts and settings never go to the remote."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-local-settings"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Settings (worktree root, target-dir layout, gc retention) live in a gitignored local config per clone, such as meta.local.toml (ignore rule committed, defaults when absent).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-local-record"),
                ArchitectureAndPlanning,
                Wording::new(
                    "The record of worktrees and target dirs (paths, creation, local trail) is typed, written only by meta, and kept inside the clone's git dir, never pushed.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-local-claim-worktree"),
                ArchitectureAndPlanning,
                Wording::new(
                    "`meta claim` creates the task's worktree under the configured root and records it. Agents make no ad-hoc clones.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-local-join"),
                ArchitectureAndPlanning,
                Wording::new("Status joins the record to the log by task id."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-local-lost-record"),
                ArchitectureAndPlanning,
                Wording::new("If the record is lost, gc deletes nothing and reports leftovers."),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Status, part 2: external state (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("status-external-live"),
                ArchitectureAndPlanning,
                Wording::new(
                    "PR and CI state is read live from GitHub on each run, with one batched query per project, and is never copied into the log (GitHub is its single home).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-external-history"),
                ArchitectureAndPlanning,
                Wording::new("History such as time red or review rounds is also derived from GitHub."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-external-unknown"),
                ArchitectureAndPlanning,
                Wording::new(
                    "If GitHub can't be reached, the state shows as unknown, and the heartbeat treats unknown as needing attention.",
                ),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Status, part 3: task-to-PR links (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("status-links-branch"),
                ArchitectureAndPlanning,
                Wording::new("`meta claim` names the task's branch."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-links-event"),
                ArchitectureAndPlanning,
                Wording::new(
                    "When a PR is opened from a claimed task's branch, the hook writes a TaskLinked { task, pr } event to the log.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-links-several"),
                ArchitectureAndPlanning,
                Wording::new("A task can link several PRs, as in a gh stack."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-links-orphan-pr"),
                ArchitectureAndPlanning,
                Wording::new("Status flags any open PR that has no task."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-links-pr-state-live"),
                ArchitectureAndPlanning,
                Wording::new("PR state itself is still read live from GitHub (part 2)."),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Status, part 4: agents, metrics and worktree lifetime (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("status-agents-independent"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Agents are an implementation detail, independent of tasks (rotating agents or dedicated pools, never 1:1); a claim becomes an assignment of agent capacity to a task.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-metrics-per-agent"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Metrics per agent, attributed to the task it is working on: (1) tokens consumed; (2) wall time; (3) compute (CPU and memory).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-metrics-wall-time"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Wall time diverges from tokens (for example a stalled script) and works at two scales: small (is this agent stalled right now?) and large (is the task on schedule against its due time?).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-metrics-compute-pool"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Compute (CPU and memory) is drawn from a central machine budget just as agents are drawn from a central pool.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-task-budgets"),
                ArchitectureAndPlanning,
                Wording::new("Task budgets: effort (tokens plus compute) and a due time set by priority."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-stalled"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Stalled means a small-scale stall, past due with no assigned capacity, or effort spent with no recorded extension.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-worktree-lifetime"),
                ArchitectureAndPlanning,
                Wording::new(
                    "A worktree belongs to a PR's branch; gc deletes it when that PR is merged, closed or stalled (resuming is cheap: recreate it from the remote), after the final check that all work is on the remote and nothing has it open.",
                ),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Status, part 5: discussion status, tokens, compute (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("status-discussion-typed"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Discussion is its own fully typed task status, not a free-form blocker: a task waiting on an owner discussion is in Discussion(issue id) and is never counted as stalled.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-waits-typed"),
                ArchitectureAndPlanning,
                Wording::new("Other waits are typed too (on another task, on something external)."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-tokens-transcripts"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Tokens are measured from Claude Code's local transcripts: each assistant message carries a usage record (input, cache, output tokens), and each subagent has its own transcript; hooks get transcript_path.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-tokens-storage"),
                ArchitectureAndPlanning,
                Wording::new("Per-task totals go to the log; raw transcripts stay local."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-compute-lease"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Compute is budgeted, not recorded: a task must hold a CPU and memory lease from a local, per-machine pool before running heavy commands (builds, test suites, mutants, Chrome checks); the pool is sized to the machine, and anything without a lease waits.",
                ),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Status, part 6: heartbeat and output (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("status-heartbeat-job"),
                ArchitectureAndPlanning,
                Wording::new(
                    "A launchd job runs `meta status` and `meta gc` every few minutes, writing flags to the local record at no token cost.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-heartbeat-wake"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Flags needing judgement (stalled, red PR with no capacity, past due, stuck command) wake a lead Claude session, which acts and goes back to sleep; for now that is the lead's scheduled wakeups.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("status-output"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Output: a terminal tree of divisions down to tasks (state, PR and CI, assigned capacity, effort against due time), with flags on top; a live dashboard page later if needed.",
                ),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Machine layer (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("machine-ownership"),
                ArchitectureAndPlanning,
                Wording::new("The machine layer is per machine and owned by no project."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("machine-config-state"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Config is in ~/.config/meta/machine.toml (CPU slots, memory, disk floor, maximum concurrent agents) and state is in ~/.local/state/meta.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("machine-lease"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Heavy commands run under meta lease (file-lock slots that a crash releases automatically, sizing CARGO_BUILD_JOBS).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("machine-hook-refusal"),
                ArchitectureAndPlanning,
                Wording::new("The hook refuses unwrapped heavy commands."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("machine-queue"),
                ArchitectureAndPlanning,
                Wording::new("Waiting commands are served by priority."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("machine-timed-runs"),
                ArchitectureAndPlanning,
                Wording::new("Timed runs take a full-machine lease."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("machine-visibility"),
                ArchitectureAndPlanning,
                Wording::new("Projects see only grant or wait; the owner sees everything."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("machine-disk-floor"),
                ArchitectureAndPlanning,
                Wording::new("Disk floor: `meta claim` refuses new worktrees below it."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("machine-order"),
                ArchitectureAndPlanning,
                Wording::new("The machine layer comes after `meta status`."),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Token measurement (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("tokens-measure-only"),
                ArchitectureAndPlanning,
                Wording::new(
                    "For now, only measure tokens per model (input, cache write, cache read, output) from local transcripts, per session and subagent, attributed to tasks.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("tokens-budgeting-deferred"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Budgeting is DEFERRED: the unit in multiples of the Claude Pro plan's limit, calibration from limit hits (first anchor 2026-10-26 about 16:10Z), and headroom.",
                ),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Commit identity guard (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("identity-noreply-only"),
                ArchitectureAndPlanning,
                Wording::new("No commit may carry an email that isn't a GitHub noreply address."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("identity-hook"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Enforced in three places, (1): the PreToolUse hook refuses git commit, and rebase or amend that rewrite commits, when the effective identity isn't noreply, naming the git config commands that fix it.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("identity-ci"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Enforced in three places, (2): cargo xtask ci (lint) fails on any commit in the PR range whose author or committer email isn't noreply.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("identity-claim"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Enforced in three places, (3): `meta claim` sets the noreply identity on worktrees it creates.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("identity-address-source"),
                ArchitectureAndPlanning,
                Wording::new("The noreply address comes from local config, not from the repo."),
                Tracked,
            ),
        ],
    ),
    Section::new(
        Wording::new("Agent pool and model tiers (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("pool-shared"),
                ArchitectureAndPlanning,
                Wording::new("Agents are drawn from one shared pool, allocated centrally."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("pool-no-nested-agents"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Implementer agents may not start subagents on their own: on 2026-10-26 nested agents and forks multiplied model usage unseen and hit the weekly limit.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("pool-allocator-spend"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Spawning should go through the allocator, and the spend (agents, tokens, wall time) should be recorded against the performance-and-cost division's budget.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("pool-spawn-hook"),
                ArchitectureAndPlanning,
                Wording::new("A PreToolUse hook on the Agent tool should refuse to spawn without a grant."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("tier-difficulty-rating"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Every task is rated by difficulty, and very mechanical tasks go to agents on smaller models.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("tier-scale"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Proposed scale: Mechanical (rename, apply a known pattern, call-site updates, named consts) to Haiku; Routine (a bounded implementation against a clear spec) to Sonnet; Hard (design, adversarial review, tricky correctness such as #7 and #8) to Opus.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("tier-typed-field"),
                ArchitectureAndPlanning,
                Wording::new("The rating is a typed field on the task that the agent-pool allocator reads."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("tier-escalation"),
                ArchitectureAndPlanning,
                Wording::new(
                    "When a cheap tier fails review or CI, the task escalates one tier and the miss is recorded, so ratings calibrate over time.",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("tier-spend"),
                ArchitectureAndPlanning,
                Wording::new("Spend is counted against the performance-and-cost budget."),
                Tracked,
            ),
        ],
    ),
];
