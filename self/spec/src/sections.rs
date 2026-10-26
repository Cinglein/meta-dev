use crate::division::Division::{ArchitectureAndPlanning, FeaturesAndMaintenance, PerformanceAndCost};
use crate::enforcement::Enforcement::Tracked;
use crate::rule::Rule;
use crate::section::Section;

pub const SECTIONS: &[Section] = &[
    Section::new(
        "Divisions (owner, 2026-10-26)",
        &[
            Rule::new(
                "division-is-a-property",
                ArchitectureAndPlanning,
                "A division is any behaviour or invariant made first class (e.g. Basho navigation). It is a named property, not a code area: divisions overlap freely in code and are isolated by tracking (own items, budgets, checks, status).",
                Tracked,
            ),
            Rule::new(
                "division-context",
                ArchitectureAndPlanning,
                "An agent on a division sees only that division's context.",
                Tracked,
            ),
            Rule::new(
                "division-nesting",
                ArchitectureAndPlanning,
                "Divisions nest arbitrarily. A child gets a subset of its parent's budget.",
                Tracked,
            ),
            Rule::new(
                "division-membership",
                ArchitectureAndPlanning,
                "Every concern, issue and task belongs to exactly one division. An unplaced item belongs to the project root; growth there signals a missing division.",
                Tracked,
            ),
            Rule::new(
                "division-top-level",
                ArchitectureAndPlanning,
                "Top level, in every project: (1) features and maintenance (navigation, the tracker, guarding lints and mutants); (2) performance and cost (CI and test time, mutation time, Actions minutes and money, disk, agent wall time); (3) architecture and planning (the spec, DISCUSS items, the ladder; feeds 1 and 2).",
                Tracked,
            ),
            Rule::new(
                "division-ownership",
                ArchitectureAndPlanning,
                "Each top-level division owns its status tracking and metrics, and controls its own soundness, testing and measurability.",
                Tracked,
            ),
            Rule::new(
                "division-dogfood",
                ArchitectureAndPlanning,
                "meta-dev is an ordinary project with the same three divisions (principle 4). The steward and triage sit in its architecture and planning division.",
                Tracked,
            ),
            Rule::new(
                "division-promotion",
                ArchitectureAndPlanning,
                "A concern that reaches ladder rung 2 (a precise property) may become a division; architecture and planning does the typing.",
                Tracked,
            ),
        ],
    ),
    Section::new(
        "Budgets (owner, 2026-10-26)",
        &[
            Rule::new(
                "budget-definition",
                PerformanceAndCost,
                "A budget is a typed definition with an id, in tooling/policy.toml. Spend events in the log point at it.",
                Tracked,
            ),
            Rule::new(
                "budget-unit",
                PerformanceAndCost,
                "Unit, exactly one per budget: Count, Duration, Share (later maybe Money or CI minutes).",
                Tracked,
            ),
            Rule::new(
                "budget-window",
                PerformanceAndCost,
                "Window: per change (ratchet against the merge base), rolling per period, or standing (open items at any time).",
                Tracked,
            ),
            Rule::new(
                "budget-overspend",
                PerformanceAndCost,
                "Overspend action is part of the definition: block (default) or escalate to DISCUSS.",
                Tracked,
            ),
            Rule::new(
                "budget-measured-spend",
                PerformanceAndCost,
                "Spend is recorded by measurement only (CI and tools). Severity is the judged exception, checked later by calibration.",
                Tracked,
            ),
            Rule::new("budget-raise", PerformanceAndCost, "Raising a budget is owner-only.", Tracked),
            Rule::new(
                "budget-waivers",
                PerformanceAndCost,
                "Waivers are spend from the waiver budget of the division whose rule they bend. They replace raw lint baselines.",
                Tracked,
            ),
        ],
    ),
    Section::new(
        "Severity per division (owner, 2026-10-26)",
        &[
            Rule::new(
                "severity-scale",
                FeaturesAndMaintenance,
                "Each division owns its severity scale. Deeper divisions have finer scales.",
                Tracked,
            ),
            Rule::new(
                "severity-mapping",
                FeaturesAndMaintenance,
                "Each child level maps to exactly one parent level, and the order is kept (a more severe child level never maps below a less severe one). Parent caps count every child item that maps to the level.",
                Tracked,
            ),
            Rule::new(
                "severity-caps",
                FeaturesAndMaintenance,
                "Caps are standing budgets per level, split down the tree. Rating into a full level needs a downgrade elsewhere or owner sign-off.",
                Tracked,
            ),
            Rule::new(
                "severity-evidence",
                FeaturesAndMaintenance,
                "A higher level needs stronger evidence (e.g. Critical: a reproduction; Major: a concrete failure scenario).",
                Tracked,
            ),
            Rule::new(
                "severity-raising",
                FeaturesAndMaintenance,
                "The reporter proposes a level. Raising after the rating is for the owner or the reviewer who reproduced it; lowering is free.",
                Tracked,
            ),
            Rule::new(
                "severity-gates-work",
                FeaturesAndMaintenance,
                "Severity orders the steward's work and gates new work (an open Critical stops new work in its division).",
                Tracked,
            ),
        ],
    ),
    Section::new(
        "Build order (2026-10-26)",
        &[
            Rule::new(
                "build-1-types",
                ArchitectureAndPlanning,
                "Step 1: Division and Budget types; policy entries with ids.",
                Tracked,
            ),
            Rule::new(
                "build-2-log-divisions",
                ArchitectureAndPlanning,
                "Step 2: every log item carries a division.",
                Tracked,
            ),
            Rule::new(
                "build-3-status",
                ArchitectureAndPlanning,
                "Step 3: `meta status` as a tree of divisions (tasks, claims, PRs and CI, agents with worktree and last activity).",
                Tracked,
            ),
            Rule::new(
                "build-4-steward",
                ArchitectureAndPlanning,
                "Step 4: steward heartbeat and `meta gc` on top of status.",
                Tracked,
            ),
            Rule::new(
                "build-5-severity",
                ArchitectureAndPlanning,
                "Step 5: severity scales, mapping and caps.",
                Tracked,
            ),
            Rule::new(
                "build-6-waivers",
                ArchitectureAndPlanning,
                "Step 6: waivers replace baselines (this also settles first-baseline seeding).",
                Tracked,
            ),
            Rule::new("build-7-rungs", ArchitectureAndPlanning, "Step 7: ladder rungs on concerns.", Tracked),
        ],
    ),
];
