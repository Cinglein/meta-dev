# Meta-development repo: spec (draft, started 2026-10-22)

Rendered from the rule records in `self/spec` down to the next top-level heading; edit those, then run `cargo run -p meta-spec`. Each rule shows id, division, rung and enforcer.

## Budgets (owner, 2026-10-26)
- `budget-definition` (architecture-and-planning, rung 1, no enforcer): A budget is a typed definition with an id, in tooling/policy.toml. Spend events in the log point at it.
- `budget-unit` (architecture-and-planning, rung 1, no enforcer): Unit, exactly one per budget: Count, Duration, Share (later maybe Money or CI minutes).
- `budget-window` (architecture-and-planning, rung 1, no enforcer): Window: per change (ratchet against the merge base), rolling per period, or standing (open items at any time).
- `budget-overspend` (architecture-and-planning, rung 1, no enforcer): Overspend action is part of the definition: block (default), report (shown, never blocks), or escalate to DISCUSS.
- `budget-measured-spend` (architecture-and-planning, rung 1, no enforcer): Spend is recorded by measurement only (CI and tools). Severity is the judged exception, checked later by calibration.
- `budget-raise` (architecture-and-planning, rung 1, no enforcer): Raising a budget is owner-only.
- `budget-waivers` (architecture-and-planning, rung 1, no enforcer): Waivers are spend from the waiver budget of the division whose rule they bend. They replace raw lint baselines.
- `budget-enforce-ci` (architecture-and-planning, rung 1, no enforcer): Budgets are enforced in CI (the local gate where Actions can't run).
- `budget-enforce-per-change` (architecture-and-planning, rung 1, no enforcer): Per-change budgets: measured fresh by CI.
- `budget-enforce-standing` (architecture-and-planning, rung 1, no enforcer): Standing budgets are derived from the log (open Criticals, waivers in force, concern backlog): CI folds meta/log read-only on every run. Every item and every PR is in exactly one division; a PR's division is its linked task's division. An over-budget standing budget blocks only PRs in its division (or a division inside it); no standing-budget path uses domains or the diff. A division's domain only scopes what its per-change budgets measure, and the code a PR touches only selects which subset of CI runs. A PR with no linked task, or whose task is at the root, is in every division and is blocked by every over-budget standing budget. A Critical elsewhere stops other divisions only through the per-change CI checks, which measure every PR.
- `budget-enforce-standing-base` (architecture-and-planning, rung 1, no enforcer): Standing budgets are enforced from the merge-base policy; the PR's own policy only has to parse and validate. A cap that the PR adds or tightens takes effect from the next PR. A cap that the PR loosens or drops still binds the PR itself, because the merge-base cap applies. The budget table marks a new cap as enforced from the next change. The merge base is the one with origin's default branch (origin/HEAD), never a branch that the PR names; a base policy that git cannot read fails the run, while a base without the file means no standing budgets. A standing cap reaches a PR only through its linked task's division, never through its diff. A PR whose task addresses an open item in the cap's division at the cap's level or more severe is paid down for that cap. A run that is not a pull request (a push to main) reports standing budgets in the table, marks the over-cap rows, and never fails: the debt rule blocks PRs, it does not turn main red.
- `budget-enforce-per-task` (architecture-and-planning, rung 1, no enforcer): Per-task budgets (effort, due): CI finds the PR's task via TaskLinked or the branch, and fails on overspend with no recorded extension; a PR with no task fails.
- `budget-enforce-spend-source` (architecture-and-planning, rung 1, no enforcer): Spend comes from local measurement (heartbeat from transcripts) written to the log; CI trusts it under the honest-mistakes threat model.
- `budget-enforce-machine-timing` (architecture-and-planning, rung 1, no enforcer): Machine budgets (CPU, memory, agents, disk) are runtime controls enforced by hooks and leases, not CI. Timing budgets are measured only on CI or in the local gate's clean run.

## Divisions (owner, 2026-10-26)
- `division-is-a-property` (architecture-and-planning, rung 1, no enforcer): A division is any behaviour or invariant made first class (e.g. Basho navigation). It is a named property, not a code area: divisions overlap freely in code and are isolated by tracking (own items, budgets, checks, status).
- `division-context` (architecture-and-planning, rung 1, no enforcer): An agent on a division sees only that division's context.
- `division-nesting` (features-and-maintenance, rung 3, check crates/meta-schema/src/division.rs a_division_id_starts_at_a_top_division_and_nests_by_slashes, check tooling/task/src/budgets.rs a_part_names_a_defined_parent_sits_inside_its_division_and_fits_its_share): Divisions nest arbitrarily. A child gets a subset of its parent's budget.
- `division-membership` (architecture-and-planning, rung 1, no enforcer): Every concern, issue and task belongs to exactly one division. An unplaced item belongs to the project root; growth there signals a missing division.
- `division-top-level` (architecture-and-planning, rung 1, no enforcer): Top level, in every project: (1) features and maintenance (navigation, the tracker, guarding lints and mutants); (2) performance and cost (CI and test time, mutation time, Actions minutes and money, disk, agent wall time); (3) architecture and planning (the spec, DISCUSS items, the ladder; feeds 1 and 2). Architecture and planning is anything not yet at the specificity of code. A code PR is never architecture. A rule moves out once it is enforced. Limits enforced in code belong to (1) or (2). A rule sits in (3) at rung 1-2, and moves to the division that owns the property once enforced (rung 3-4).
- `division-ownership` (architecture-and-planning, rung 1, no enforcer): Each top-level division owns its status tracking and metrics, and controls its own soundness, testing and measurability.
- `division-dogfood` (architecture-and-planning, rung 1, no enforcer): meta-dev is an ordinary project with the same three divisions (principle 4). The steward and triage sit in its architecture and planning division.
- `division-promotion` (architecture-and-planning, rung 1, no enforcer): Promotion: a concern that reaches ladder rung 2 (a precise property) may become a division; architecture and planning does the typing.
- `domain-declared` (architecture-and-planning, rung 1, no enforcer): Only what is measured can be budgeted, so every division declares its domain as typed scopes in policy: crates, paths, CI workflows, tests, plus its implicit items (log entries labelled with it).
- `domain-subset` (features-and-maintenance, rung 3, check tooling/task/src/domains.rs domains_outside_their_parent_planning_code_unmeasurable_budgets_and_unordered_scales_are_refused): A child's domain is a strict subset of its parent's.
- `domain-defaults` (features-and-maintenance, rung 3, check tooling/task/src/domains.rs by_default_ci_owns_tests_and_workflows_code_owns_crates_and_paths_and_planning_nothing): Defaults: all code belongs to features-and-maintenance and, through CI performance (which is always measured, unlike app performance), to performance-and-cost. Architecture-and-planning has no code.
- `domain-budget-measure` (architecture-and-planning, rung 1, no enforcer): A budget is a measure over its division's domain (test or workflow wall time, lint findings over crates or paths, open Criticals over items). A budget whose measure needs a scope the division doesn't declare is refused when the policy is read.
- `domain-reach` (features-and-maintenance, rung 3, check tooling/task/src/budgets/ledger.rs an_over_standing_budget_blocks_a_change_linked_inside_its_division_unless_it_pays_down, check tooling/task/src/budgets/ledger.rs an_overspend_fails_a_change_within_reach_and_a_nested_part_is_measured): A per-change budget reaches a PR when its diff touches the domain; the domain only scopes what per-change budgets measure. A standing budget never uses the domain or the diff: it reaches a PR only when its linked task is in the division; a PR with no linked task, or whose task is at the root, is within every standing budget's reach.
- `domain-scope-kinds` (architecture-and-planning, rung 1, no enforcer): Which scope kinds a budget needs depends on the budget; wall-time budgets on tests and workflows are the easiest first case.
- `concern-definition` (architecture-and-planning, rung 1, no enforcer): A concern is anything noticed that might matter, vague or precise, and is the only entity that carries a ladder rung.
- `concern-rungs` (architecture-and-planning, rung 1, no enforcer): Rungs: 1 identified (text); 2 defined (points at a spec rule); 3 checked or budgeted (a named enforcer: lint, test or CI step); 4 impossible by construction (a type).
- `concern-moves` (architecture-and-planning, rung 1, no enforcer): Moves up and down are events with evidence; a concern never claims a rung its evidence no longer supports.
- `concern-division` (architecture-and-planning, rung 1, no enforcer): A concern's division is architecture-and-planning at rungs 1-2, then the owning division at 3-4.
- `concern-resting` (architecture-and-planning, rung 1, no enforcer): Concerns may rest at any rung, but resting is budgeted with a ratchet (exact shape being decided).
- `concern-rung-only-axis` (architecture-and-planning, rung 1, no enforcer): Rung is the only typed axis; confidence is not separate, because climbing from rung 1 to 2 already narrows it.
- `concern-closing` (features-and-maintenance, rung 3, check crates/meta-state/src/state/tests.rs done_addresses_targets_and_reopen_undoes_it, check crates/meta-state/src/state/tests/rating.rs retired_and_unknown_items_are_not_rated_and_only_open_items_are_listed): Closing: resolved or retired.
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
- `status-local-join` (features-and-maintenance, rung 3, check crates/meta-cli/src/request/local/tests.rs worktrees_join_the_current_trail_to_task_status_and_flag_missing_and_unrecorded_paths): Status joins the record to the log by task id.
- `status-local-lost-record` (features-and-maintenance, rung 3, check crates/meta-cli/tests/gc.rs gc_frees_ended_pushed_worktrees_with_their_target_dirs_and_keeps_the_rest_with_reasons, check crates/meta-cli/src/request/local/tests.rs settings_default_to_a_sibling_root_resolve_relative_roots_and_refuse_foreign_emails): If the record is lost, gc deletes nothing and reports leftovers.

## Status, part 2: external state (owner, 2026-10-26)
- `status-external-live` (architecture-and-planning, rung 1, no enforcer): PR and CI state is read live from GitHub on each run, with one batched query per project, and is never copied into the log (GitHub is its single home).
- `status-external-history` (architecture-and-planning, rung 1, no enforcer): History such as time red or review rounds is also derived from GitHub.
- `status-external-unknown` (features-and-maintenance, rung 3, check crates/meta-status/src/status/tests.rs an_unreachable_github_is_one_flag_and_every_pull_request_reads_unknown, check crates/meta-status/src/flags/tests.rs only_orphans_never_await_judgement_and_flags_survive_a_round_trip_through_json, check crates/meta-cli/src/request/tests/heartbeat/tick.rs a_tick_prints_only_new_flags_and_exits_non_zero_only_for_flags_needing_judgement): If GitHub can't be reached, the state shows as unknown, and the heartbeat treats unknown as needing attention.

## Status, part 3: task-to-PR links (owner, 2026-10-26)
- `status-links-branch` (features-and-maintenance, rung 3, check crates/meta-cli/tests/cli.rs failed_worktrees_leave_no_claim_and_a_held_task_gets_its_lost_worktree_back_and_a_noreply_worktree_is_listed): `meta claim` names the task's branch.
- `status-links-event` (features-and-maintenance, rung 3, check crates/meta-cli/tests/status.rs opening_a_pull_request_from_a_claimed_tasks_branch_links_it_and_status_flags_the_rest): When a PR is opened from a claimed task's branch, the hook writes a TaskLinked { task, pr } event to the log.
- `status-links-several` (features-and-maintenance, rung 3, check crates/meta-state/src/state/tests/links.rs a_task_links_several_pull_requests_once_each_and_only_known_tasks_link): A task can link several PRs, as in a gh stack.
- `status-links-orphan-pr` (features-and-maintenance, rung 3, check crates/meta-status/src/status/tests.rs flags_name_late_red_unknown_truncated_and_orphan_pull_requests_and_red_ones_on_done_tasks, check crates/meta-pull-query/src/live_tests.rs open_pull_requests_read_as_orphans_and_truncation_is_reported): Status flags any open PR that has no task.
- `status-links-pr-state-live` (features-and-maintenance, rung 3, check crates/meta-pull-query/src/live_tests.rs the_query_asks_for_open_pull_requests_and_each_linked_one_once, check crates/meta-cli/src/request/live_github/tests.rs one_graphql_call_then_the_base_rules_read_a_partial_reply_and_a_branch_lists_its_own_pull_requests): PR state itself is still read live from GitHub (part 2).

## Status, part 4: agents, metrics and worktree lifetime (owner, 2026-10-26)
- `status-agents-independent` (architecture-and-planning, rung 1, no enforcer): Agents are an implementation detail, independent of tasks (rotating agents or dedicated pools, never 1:1); a claim becomes an assignment of agent capacity to a task.
- `status-metrics-per-agent` (architecture-and-planning, rung 1, no enforcer): Metrics per agent, attributed to the task it is working on: (1) tokens consumed; (2) wall time; (3) compute (CPU and memory).
- `status-metrics-wall-time` (architecture-and-planning, rung 1, no enforcer): Wall time diverges from tokens (for example a stalled script) and works at two scales: small (is this agent stalled right now?) and large (is the task on schedule against its due time?).
- `status-metrics-compute-pool` (architecture-and-planning, rung 1, no enforcer): Compute (CPU and memory) is drawn from a central machine budget just as agents are drawn from a central pool.
- `status-task-budgets` (architecture-and-planning, rung 1, no enforcer): Task budgets: effort (tokens plus compute) and a due time set by priority.
- `status-stalled` (architecture-and-planning, rung 1, no enforcer): Stalled means a small-scale stall, past due with no assigned capacity, or effort spent with no recorded extension.
- `status-worktree-lifetime` (features-and-maintenance, rung 3, check crates/meta-status/src/status/lifetime_tests.rs a_task_ends_when_every_linked_pull_request_is_merged_or_closed_or_it_stalls_and_unknown_never_ends, check crates/meta-cli/tests/gc.rs gc_frees_ended_pushed_worktrees_with_their_target_dirs_and_keeps_the_rest_with_reasons, check crates/meta-cli/src/request/local/local_clone/vetting/tests.rs only_open_files_under_the_worktree_block_it_and_a_failed_scan_blocks_every_worktree, check crates/meta-cli/tests/cli.rs failed_worktrees_leave_no_claim_and_a_held_task_gets_its_lost_worktree_back_and_a_noreply_worktree_is_listed): A worktree belongs to a PR's branch; gc deletes it when that PR is merged, closed or stalled (resuming is cheap: recreate it from the remote), after the final check that all work is on the remote and nothing has it open.

## Status, part 5: discussion status, tokens, compute (owner, 2026-10-26)
- `status-discussion-typed` (features-and-maintenance, rung 3, check crates/meta-state/src/state/tests/blocking.rs only_discussion_never_stalls_and_every_wait_is_a_typed_status): Discussion is its own fully typed task status, not a free-form blocker: a task waiting on an owner discussion is in Discussion(issue id) and is never counted as stalled.
- `status-waits-typed` (features-and-maintenance, rung 3, check crates/meta-state/src/state/tests/blocking.rs only_discussion_never_stalls_and_every_wait_is_a_typed_status, check crates/meta-schema/src/parsing_tests.rs a_blocker_reads_as_an_issue_a_task_or_an_external_kind_and_nothing_else): Other waits are typed too (on another task, on something external).
- `status-tokens-transcripts` (architecture-and-planning, rung 1, no enforcer): Tokens are measured from Claude Code's local transcripts: each assistant message carries a usage record (input, cache, output tokens), and each subagent has its own transcript; hooks get transcript_path.
- `status-tokens-storage` (architecture-and-planning, rung 1, no enforcer): Per-task totals go to the log; raw transcripts stay local.
- `status-compute-lease` (architecture-and-planning, rung 1, no enforcer): Compute is budgeted, not recorded: a task must hold a CPU and memory lease from a local, per-machine pool before running heavy commands (builds, test suites, mutants, Chrome checks); the pool is sized to the machine, and anything without a lease waits.

## Status, part 6: heartbeat and output (owner, 2026-10-26)
- `status-heartbeat-job` (architecture-and-planning, rung 1, no enforcer): A launchd job runs `meta status` and `meta gc` every few minutes, writing flags to the local record at no token cost.
- `status-heartbeat-wake` (architecture-and-planning, rung 1, no enforcer): Flags needing judgement (stalled, red PR with no capacity, past due, stuck command) wake a lead Claude session, which acts and goes back to sleep; for now that is the lead's scheduled wakeups.
- `status-output` (features-and-maintenance, rung 3, check crates/meta-status/src/status/tests.rs the_tree_runs_from_divisions_down_to_tasks_with_pull_requests_holders_effort_and_due): Output: a terminal tree of divisions down to tasks (state, PR and CI, assigned capacity, effort against due time), with flags on top; a live dashboard page later if needed.

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
- `identity-noreply-only` (features-and-maintenance, rung 3, check xtask/src/task/lint.rs every_commit_off_the_base_branch_has_noreply_author_and_committer_emails): No commit may carry an email that isn't a GitHub noreply address.
- `identity-hook` (architecture-and-planning, rung 1, no enforcer): Enforced in three places, (1): the PreToolUse hook refuses git commit, and rebase or amend that rewrite commits, when the effective identity isn't noreply, naming the git config commands that fix it.
- `identity-ci` (features-and-maintenance, rung 3, check xtask/src/task/lint.rs every_commit_off_the_base_branch_has_noreply_author_and_committer_emails): Enforced in three places, (2): cargo xtask ci (lint) fails on any commit in the PR range whose author or committer email isn't noreply.
- `identity-claim` (features-and-maintenance, rung 3, check crates/meta-cli/tests/cli.rs failed_worktrees_leave_no_claim_and_a_held_task_gets_its_lost_worktree_back_and_a_noreply_worktree_is_listed): Enforced in three places, (3): `meta claim` sets the noreply identity on worktrees it creates.
- `identity-address-source` (features-and-maintenance, rung 3, check crates/meta-cli/src/request/local/tests.rs settings_default_to_a_sibling_root_resolve_relative_roots_and_refuse_foreign_emails, check crates/meta-cli/tests/cli.rs failed_worktrees_leave_no_claim_and_a_held_task_gets_its_lost_worktree_back_and_a_noreply_worktree_is_listed): The noreply address comes from local config, not from the repo.

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

## Decisions (owner, 2026-10-27)
- `ci-tracked-work` (features-and-maintenance, rung 3, check xtask/src/task/ci/tracked_work.rs no_branch_no_task_unknown_retired_or_unclaimed_tasks_and_an_unreadable_log_fail_with_the_remedy, check xtask/src/task/ci/tracked_work.rs claimed_or_done_task_branches_pass_runs_without_a_pull_request_signal_read_nothing_and_any_signal_is_a_pull): CI fails a pull request whose work is not tracked in meta: its branch must name a claimed or done task (task-74151ea135505567).
- `hook-claimed-writes` (architecture-and-planning, rung 1, no enforcer): Agent hooks refuse file writes and commits outside a claimed task (task-0bd3aeb5a7081bea).
- `hook-task-injected` (architecture-and-planning, rung 1, no enforcer): Agent hooks inject the branch's task and a short status at session and subagent start (task-0bd3aeb5a7081bea).
- `monitor-report-only` (performance-and-cost, rung 3, check crates/meta-cli/src/request/local/monitor/tests.rs by_default_the_monitor_signals_nothing_and_reports_this_projects_processes_in_two_tiers, check crates/meta-cli/src/request/local/monitor/tests.rs opted_in_stopping_spares_the_owner_shell_claude_its_children_other_projects_and_a_mixed_group): The monitor reports this project's processes in two tiers (obvious, watch) and stops nothing unless stopping is opted in (task-342fa1089f956572).
- `monitor-gc-disk` (architecture-and-planning, rung 1, no enforcer): The monitor reports disk per agent folder, and gc deletes target dirs freely and a finished task's scratch and worktrees (task-c4c1e9b122d7d515).
- `init-rust-idempotent` (architecture-and-planning, rung 1, no enforcer): `meta init` adopts Rust projects only and is idempotent: a set-up project is a no-op and a partial one gets only its missing steps (task-cdf6aa4887922906).
- `init-explicit-keys` (architecture-and-planning, rung 1, no enforcer): `meta init` and `meta upgrade` write every file and policy key explicitly; no default stays hidden in code (task-2fcb95cb7eb36cdf).
- `init-baseline-seeding` (architecture-and-planning, rung 1, no enforcer): CI allows the lint baseline to be created once, in the adoption PR whose merge base has no meta setup; later PRs may only shrink it (task-f72f1ea5220a3499).
- `init-ruleset-confirmed` (architecture-and-planning, rung 1, no enforcer): `meta init` applies the main ruleset only after the owner confirms it; run by an agent, it prints the command instead (task-e20cb6a118fb3bea).
- `init-actions-gate` (architecture-and-planning, rung 1, no enforcer): The required gate is the GitHub Actions ci-gate by default; switching to the local gate is explicit and visible (task-29e18872772ed789).
- `init-existing-ci` (architecture-and-planning, rung 1, no enforcer): `meta init` adds its CI job beside a project's existing jobs and makes ci-gate need them all, so adoption never loosens a check (task-3802fb69b5255061).
- `meta-pinned-xtask` (architecture-and-planning, rung 1, no enforcer): Hooks, the monitor and agents run meta through the project's pinned xtask; there is no global install (task-4a2147350c188a12).
- `consumers-meta-xtask-only` (architecture-and-planning, rung 1, no enforcer): Consumers depend only on meta-xtask; internal tooling crates are no one's API (task-3d06e7b8efda4539).
- `meta-upgrade` (architecture-and-planning, rung 1, no enforcer): `meta upgrade` moves the meta-dev pin and rewrites the setup with the init writer, adding new policy keys with their defaults (task-cdf6aa4887922906, task-4a2147350c188a12).
- `dispatch-route-b` (architecture-and-planning, rung 1, no enforcer): Subagents go through `meta dispatch` in hooks: none without a claimed task, and each gets the rules block, a tier and a lease (task-a940d57cf475d814).
- `standing-merge-base-policy` (architecture-and-planning, rung 1, no enforcer): Standing budgets on a pull request are enforced from the merge-base policy; on other runs they are only reported (task-bc3f3a49821699b0).
- `acceptance-list-current` (architecture-and-planning, rung 1, no enforcer): The acceptance list in self/spec stays current: every postmortem lesson is an entry, met (naming its enforcer) or pending (naming its open concern) (task-54d4acc03dfdc35c).
- `spec-enforcer-exists` (features-and-maintenance, rung 3, check self/spec/src/spec_view.rs the_checked_in_spec_passes_its_check_rule_ids_are_unique_and_a_lost_enforcer_is_named, check self/spec/src/enforcement.rs each_rung_shows_its_enforcer_and_a_check_is_found_only_by_its_name_in_its_file): CI fails when a rule names an enforcer that does not exist (task-adbcd43d30f4f41b).

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
