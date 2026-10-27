use crate::enforcement::Enforcement::{Check, Tracked};
use crate::enforcer::Enforcer;
use crate::rule::{Rule, RuleId};
use crate::section::Section;
use crate::wording::Wording;
use task::TopDivision::{ArchitectureAndPlanning, FeaturesAndMaintenance, PerformanceAndCost};

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
                FeaturesAndMaintenance,
                Wording::new("Divisions nest arbitrarily. A child gets a subset of its parent's budget."),
                Check(
                    Enforcer::new(
                        "crates/meta-schema/src/division.rs",
                        "a_division_id_starts_at_a_top_division_and_nests_by_slashes",
                    ),
                    &[Enforcer::new(
                        "tooling/task/src/budgets.rs",
                        "a_part_names_a_defined_parent_sits_inside_its_division_and_fits_its_share",
                    )],
                ),
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
                FeaturesAndMaintenance,
                Wording::new("A child's domain is a strict subset of its parent's."),
                Check(
                    Enforcer::new(
                        "tooling/task/src/domains.rs",
                        "domains_outside_their_parent_planning_code_unmeasurable_budgets_and_unordered_scales_are_refused",
                    ),
                    &[],
                ),
            ),
            Rule::new(
                RuleId::new("domain-defaults"),
                FeaturesAndMaintenance,
                Wording::new(
                    "Defaults: all code belongs to features-and-maintenance and, through CI performance (which is always measured, unlike app performance), to performance-and-cost. Architecture-and-planning has no code.",
                ),
                Check(
                    Enforcer::new(
                        "tooling/task/src/domains.rs",
                        "by_default_ci_owns_tests_and_workflows_code_owns_crates_and_paths_and_planning_nothing",
                    ),
                    &[],
                ),
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
                FeaturesAndMaintenance,
                Wording::new(
                    "A PR is within a budget's reach when its diff touches the domain or its linked task is in the division.",
                ),
                Check(
                    Enforcer::new(
                        "tooling/task/src/budgets/ledger.rs",
                        "an_over_standing_budget_blocks_a_change_linked_inside_its_division_unless_it_pays_down",
                    ),
                    &[Enforcer::new(
                        "tooling/task/src/budgets/ledger.rs",
                        "an_overspend_fails_a_change_within_reach_and_a_nested_part_is_measured",
                    )],
                ),
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
                FeaturesAndMaintenance,
                Wording::new("Closing: resolved or retired."),
                Check(
                    Enforcer::new(
                        "crates/meta-state/src/state/tests.rs",
                        "done_addresses_targets_and_reopen_undoes_it",
                    ),
                    &[Enforcer::new(
                        "crates/meta-state/src/state/tests/rating.rs",
                        "retired_and_unknown_items_are_not_rated_and_only_open_items_are_listed",
                    )],
                ),
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
                FeaturesAndMaintenance,
                Wording::new("Status joins the record to the log by task id."),
                Check(
                    Enforcer::new(
                        "crates/meta-cli/src/request/local/tests.rs",
                        "worktrees_join_the_current_trail_to_task_status_and_flag_missing_and_unrecorded_paths",
                    ),
                    &[],
                ),
            ),
            Rule::new(
                RuleId::new("status-local-lost-record"),
                FeaturesAndMaintenance,
                Wording::new("If the record is lost, gc deletes nothing and reports leftovers."),
                Check(
                    Enforcer::new(
                        "crates/meta-cli/tests/gc.rs",
                        "gc_frees_ended_pushed_worktrees_with_their_target_dirs_and_keeps_the_rest_with_reasons",
                    ),
                    &[Enforcer::new(
                        "crates/meta-cli/src/request/local/tests.rs",
                        "settings_default_to_a_sibling_root_resolve_relative_roots_and_refuse_foreign_emails",
                    )],
                ),
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
                FeaturesAndMaintenance,
                Wording::new(
                    "If GitHub can't be reached, the state shows as unknown, and the heartbeat treats unknown as needing attention.",
                ),
                Check(
                    Enforcer::new(
                        "crates/meta-status/src/status/tests.rs",
                        "an_unreachable_github_is_one_flag_and_every_pull_request_reads_unknown",
                    ),
                    &[
                        Enforcer::new(
                            "crates/meta-status/src/flags/tests.rs",
                            "only_orphans_never_await_judgement_and_flags_survive_a_round_trip_through_json",
                        ),
                        Enforcer::new(
                            "crates/meta-cli/src/request/tests/heartbeat/tick.rs",
                            "a_tick_prints_only_new_flags_and_exits_non_zero_only_for_flags_needing_judgement",
                        ),
                    ],
                ),
            ),
        ],
    ),
    Section::new(
        Wording::new("Status, part 3: task-to-PR links (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("status-links-branch"),
                FeaturesAndMaintenance,
                Wording::new("`meta claim` names the task's branch."),
                Check(
                    Enforcer::new(
                        "crates/meta-cli/tests/cli.rs",
                        "failed_worktrees_leave_no_claim_and_a_held_task_gets_its_lost_worktree_back_and_a_noreply_worktree_is_listed",
                    ),
                    &[],
                ),
            ),
            Rule::new(
                RuleId::new("status-links-event"),
                FeaturesAndMaintenance,
                Wording::new(
                    "When a PR is opened from a claimed task's branch, the hook writes a TaskLinked { task, pr } event to the log.",
                ),
                Check(
                    Enforcer::new(
                        "crates/meta-cli/tests/status.rs",
                        "opening_a_pull_request_from_a_claimed_tasks_branch_links_it_and_status_flags_the_rest",
                    ),
                    &[],
                ),
            ),
            Rule::new(
                RuleId::new("status-links-several"),
                FeaturesAndMaintenance,
                Wording::new("A task can link several PRs, as in a gh stack."),
                Check(
                    Enforcer::new(
                        "crates/meta-state/src/state/tests/links.rs",
                        "a_task_links_several_pull_requests_once_each_and_only_known_tasks_link",
                    ),
                    &[],
                ),
            ),
            Rule::new(
                RuleId::new("status-links-orphan-pr"),
                FeaturesAndMaintenance,
                Wording::new("Status flags any open PR that has no task."),
                Check(
                    Enforcer::new(
                        "crates/meta-status/src/status/tests.rs",
                        "flags_name_late_red_unknown_truncated_and_orphan_pull_requests_and_red_ones_on_done_tasks",
                    ),
                    &[Enforcer::new(
                        "crates/meta-pull-query/src/live_tests.rs",
                        "open_pull_requests_read_as_orphans_and_truncation_is_reported",
                    )],
                ),
            ),
            Rule::new(
                RuleId::new("status-links-pr-state-live"),
                FeaturesAndMaintenance,
                Wording::new("PR state itself is still read live from GitHub (part 2)."),
                Check(
                    Enforcer::new(
                        "crates/meta-pull-query/src/live_tests.rs",
                        "the_query_asks_for_open_pull_requests_and_each_linked_one_once",
                    ),
                    &[Enforcer::new(
                        "crates/meta-cli/src/request/live_github/tests.rs",
                        "one_graphql_call_then_the_base_rules_read_a_partial_reply_and_a_branch_lists_its_own_pull_requests",
                    )],
                ),
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
                FeaturesAndMaintenance,
                Wording::new(
                    "A worktree belongs to a PR's branch; gc deletes it when that PR is merged, closed or stalled (resuming is cheap: recreate it from the remote), after the final check that all work is on the remote and nothing has it open.",
                ),
                Check(
                    Enforcer::new(
                        "crates/meta-status/src/status/lifetime_tests.rs",
                        "a_task_ends_when_every_linked_pull_request_is_merged_or_closed_or_it_stalls_and_unknown_never_ends",
                    ),
                    &[
                        Enforcer::new(
                            "crates/meta-cli/tests/gc.rs",
                            "gc_frees_ended_pushed_worktrees_with_their_target_dirs_and_keeps_the_rest_with_reasons",
                        ),
                        Enforcer::new(
                            "crates/meta-cli/src/request/local/local_clone/vetting/tests.rs",
                            "only_open_files_under_the_worktree_block_it_and_a_failed_scan_blocks_every_worktree",
                        ),
                        Enforcer::new(
                            "crates/meta-cli/tests/cli.rs",
                            "failed_worktrees_leave_no_claim_and_a_held_task_gets_its_lost_worktree_back_and_a_noreply_worktree_is_listed",
                        ),
                    ],
                ),
            ),
        ],
    ),
    Section::new(
        Wording::new("Status, part 5: discussion status, tokens, compute (owner, 2026-10-26)"),
        &[
            Rule::new(
                RuleId::new("status-discussion-typed"),
                FeaturesAndMaintenance,
                Wording::new(
                    "Discussion is its own fully typed task status, not a free-form blocker: a task waiting on an owner discussion is in Discussion(issue id) and is never counted as stalled.",
                ),
                Check(
                    Enforcer::new(
                        "crates/meta-state/src/state/tests/blocking.rs",
                        "only_discussion_never_stalls_and_every_wait_is_a_typed_status",
                    ),
                    &[],
                ),
            ),
            Rule::new(
                RuleId::new("status-waits-typed"),
                FeaturesAndMaintenance,
                Wording::new("Other waits are typed too (on another task, on something external)."),
                Check(
                    Enforcer::new(
                        "crates/meta-state/src/state/tests/blocking.rs",
                        "only_discussion_never_stalls_and_every_wait_is_a_typed_status",
                    ),
                    &[Enforcer::new(
                        "crates/meta-schema/src/parsing_tests.rs",
                        "a_blocker_reads_as_an_issue_a_task_or_an_external_kind_and_nothing_else",
                    )],
                ),
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
                FeaturesAndMaintenance,
                Wording::new(
                    "Output: a terminal tree of divisions down to tasks (state, PR and CI, assigned capacity, effort against due time), with flags on top; a live dashboard page later if needed.",
                ),
                Check(
                    Enforcer::new(
                        "crates/meta-status/src/status/tests.rs",
                        "the_tree_runs_from_divisions_down_to_tasks_with_pull_requests_holders_effort_and_due",
                    ),
                    &[],
                ),
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
                FeaturesAndMaintenance,
                Wording::new("No commit may carry an email that isn't a GitHub noreply address."),
                Check(
                    Enforcer::new(
                        "xtask/src/task/lint.rs",
                        "every_commit_off_the_base_branch_has_noreply_author_and_committer_emails",
                    ),
                    &[],
                ),
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
                FeaturesAndMaintenance,
                Wording::new(
                    "Enforced in three places, (2): cargo xtask ci (lint) fails on any commit in the PR range whose author or committer email isn't noreply.",
                ),
                Check(
                    Enforcer::new(
                        "xtask/src/task/lint.rs",
                        "every_commit_off_the_base_branch_has_noreply_author_and_committer_emails",
                    ),
                    &[],
                ),
            ),
            Rule::new(
                RuleId::new("identity-claim"),
                FeaturesAndMaintenance,
                Wording::new(
                    "Enforced in three places, (3): `meta claim` sets the noreply identity on worktrees it creates.",
                ),
                Check(
                    Enforcer::new(
                        "crates/meta-cli/tests/cli.rs",
                        "failed_worktrees_leave_no_claim_and_a_held_task_gets_its_lost_worktree_back_and_a_noreply_worktree_is_listed",
                    ),
                    &[],
                ),
            ),
            Rule::new(
                RuleId::new("identity-address-source"),
                FeaturesAndMaintenance,
                Wording::new("The noreply address comes from local config, not from the repo."),
                Check(
                    Enforcer::new(
                        "crates/meta-cli/src/request/local/tests.rs",
                        "settings_default_to_a_sibling_root_resolve_relative_roots_and_refuse_foreign_emails",
                    ),
                    &[Enforcer::new(
                        "crates/meta-cli/tests/cli.rs",
                        "failed_worktrees_leave_no_claim_and_a_held_task_gets_its_lost_worktree_back_and_a_noreply_worktree_is_listed",
                    )],
                ),
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
    Section::new(
        Wording::new("Decisions (owner, 2026-10-27)"),
        &[
            Rule::new(
                RuleId::new("ci-tracked-work"),
                FeaturesAndMaintenance,
                Wording::new(
                    "CI fails a pull request whose work is not tracked in meta: its branch must name a claimed or done task (task-74151ea135505567).",
                ),
                Check(
                    Enforcer::new(
                        "xtask/src/task/ci/tracked_work.rs",
                        "no_branch_no_task_unknown_retired_or_unclaimed_tasks_and_an_unreadable_log_fail_with_the_remedy",
                    ),
                    &[Enforcer::new(
                        "xtask/src/task/ci/tracked_work.rs",
                        "claimed_or_done_task_branches_pass_runs_without_a_pull_request_signal_read_nothing_and_any_signal_is_a_pull",
                    )],
                ),
            ),
            Rule::new(
                RuleId::new("hook-claimed-writes"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Agent hooks refuse file writes and commits outside a claimed task (task-0bd3aeb5a7081bea).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("hook-task-injected"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Agent hooks inject the branch's task and a short status at session and subagent start (task-0bd3aeb5a7081bea).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("monitor-report-only"),
                PerformanceAndCost,
                Wording::new(
                    "The monitor reports this project's processes in two tiers (obvious, watch) and stops nothing unless stopping is opted in (task-342fa1089f956572).",
                ),
                Check(
                    Enforcer::new(
                        "crates/meta-cli/src/request/local/monitor/tests.rs",
                        "by_default_the_monitor_signals_nothing_and_reports_this_projects_processes_in_two_tiers",
                    ),
                    &[Enforcer::new(
                        "crates/meta-cli/src/request/local/monitor/tests.rs",
                        "opted_in_stopping_spares_the_owner_shell_claude_its_children_other_projects_and_a_mixed_group",
                    )],
                ),
            ),
            Rule::new(
                RuleId::new("monitor-gc-disk"),
                ArchitectureAndPlanning,
                Wording::new(
                    "The monitor reports disk per agent folder, and gc deletes target dirs freely and a finished task's scratch and worktrees (task-c4c1e9b122d7d515).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("init-rust-idempotent"),
                ArchitectureAndPlanning,
                Wording::new(
                    "`meta init` adopts Rust projects only and is idempotent: a set-up project is a no-op and a partial one gets only its missing steps (task-cdf6aa4887922906).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("init-explicit-keys"),
                ArchitectureAndPlanning,
                Wording::new(
                    "`meta init` and `meta upgrade` write every file and policy key explicitly; no default stays hidden in code (task-2fcb95cb7eb36cdf).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("init-baseline-seeding"),
                ArchitectureAndPlanning,
                Wording::new(
                    "CI allows the lint baseline to be created once, in the adoption PR whose merge base has no meta setup; later PRs may only shrink it (task-f72f1ea5220a3499).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("init-ruleset-confirmed"),
                ArchitectureAndPlanning,
                Wording::new(
                    "`meta init` applies the main ruleset only after the owner confirms it; run by an agent, it prints the command instead (task-e20cb6a118fb3bea).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("init-actions-gate"),
                ArchitectureAndPlanning,
                Wording::new(
                    "The required gate is the GitHub Actions ci-gate by default; switching to the local gate is explicit and visible (task-29e18872772ed789).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("init-existing-ci"),
                ArchitectureAndPlanning,
                Wording::new(
                    "`meta init` adds its CI job beside a project's existing jobs and makes ci-gate need them all, so adoption never loosens a check (task-3802fb69b5255061).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("meta-pinned-xtask"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Hooks, the monitor and agents run meta through the project's pinned xtask; there is no global install (task-4a2147350c188a12).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("consumers-meta-xtask-only"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Consumers depend only on meta-xtask; internal tooling crates are no one's API (task-3d06e7b8efda4539).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("meta-upgrade"),
                ArchitectureAndPlanning,
                Wording::new(
                    "`meta upgrade` moves the meta-dev pin and rewrites the setup with the init writer, adding new policy keys with their defaults (task-cdf6aa4887922906, task-4a2147350c188a12).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("dispatch-route-b"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Subagents go through `meta dispatch` in hooks: none without a claimed task, and each gets the rules block, a tier and a lease (task-a940d57cf475d814).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("standing-merge-base-policy"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Standing budgets on a pull request are enforced from the merge-base policy; on other runs they are only reported (task-bc3f3a49821699b0).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("acceptance-list-current"),
                ArchitectureAndPlanning,
                Wording::new(
                    "The acceptance list in self/spec stays current: every postmortem lesson is an entry, met (naming its enforcer) or pending (naming its open concern) (task-54d4acc03dfdc35c).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("spec-enforcer-exists"),
                FeaturesAndMaintenance,
                Wording::new("CI fails when a rule names an enforcer that does not exist (task-adbcd43d30f4f41b)."),
                Check(
                    Enforcer::new(
                        "self/spec/src/spec_view.rs",
                        "the_checked_in_spec_passes_its_check_rule_ids_are_unique_and_a_lost_enforcer_is_named",
                    ),
                    &[Enforcer::new(
                        "self/spec/src/enforcement.rs",
                        "each_rung_shows_its_enforcer_and_a_check_is_found_only_by_its_name_in_its_file",
                    )],
                ),
            ),
        ],
    ),
];
