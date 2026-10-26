use crate::enforcement::Enforcement::Tracked;
use crate::rule::{Rule, RuleId};
use crate::section::Section;
use crate::wording::Wording;
use task::TopDivision::ArchitectureAndPlanning;

pub const SECTIONS: &[Section] = &[
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
                    "Top level, in every project: (1) features and maintenance (navigation, the tracker, guarding lints and mutants); (2) performance and cost (CI and test time, mutation time, Actions minutes and money, disk, agent wall time); (3) architecture and planning (the spec, DISCUSS items, the ladder; feeds 1 and 2).",
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
        ],
    ),
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
                Wording::new("Step 2: every log item carries a division."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-3-status"),
                ArchitectureAndPlanning,
                Wording::new(
                    "Step 3: `meta status` as a tree of divisions (tasks, claims, PRs and CI, agents with worktree and last activity).",
                ),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-4-steward"),
                ArchitectureAndPlanning,
                Wording::new("Step 4: steward heartbeat and `meta gc` on top of status."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-5-severity"),
                ArchitectureAndPlanning,
                Wording::new("Step 5: severity scales, mapping and caps."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-6-waivers"),
                ArchitectureAndPlanning,
                Wording::new("Step 6: waivers replace baselines (this also settles first-baseline seeding)."),
                Tracked,
            ),
            Rule::new(
                RuleId::new("build-7-rungs"),
                ArchitectureAndPlanning,
                Wording::new("Step 7: ladder rungs on concerns."),
                Tracked,
            ),
        ],
    ),
];
