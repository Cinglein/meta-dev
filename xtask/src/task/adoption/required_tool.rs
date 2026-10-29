use strum::{Display, EnumIter};
use task::{FullRun, Measure, Policy};

use crate::task::Task;
use crate::task::adoption::gap::Gap;
use crate::task::adoption::trigger::Trigger;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum RequiredTool {
    Ci,
    CargoDeny,
    CargoSemverChecks,
    DiffMutants,
    FullMutants,
    Stacked,
    TestCpuTimeBudget,
    DependabotUpdates,
    DependabotAlerts,
}

impl RequiredTool {
    pub fn subcommand(self) -> Option<&'static str> {
        match self {
            RequiredTool::CargoDeny => Some("deny"),
            RequiredTool::CargoSemverChecks => Some("semver-checks"),
            _ => None,
        }
    }

    pub fn task(self) -> Option<Task> {
        match self {
            RequiredTool::Ci => Some(Task::Ci),
            RequiredTool::DiffMutants => Some(Task::Mutants),
            RequiredTool::FullMutants => Some(Task::MutantsFull),
            RequiredTool::Stacked => Some(Task::Stacked),
            _ => None,
        }
    }

    pub fn trigger(self) -> Trigger {
        match self {
            RequiredTool::FullMutants => Trigger::Main,
            _ => Trigger::PullRequest,
        }
    }

    pub fn turned_off(self, policy: &Policy) -> Option<Gap> {
        match self {
            RequiredTool::CargoSemverChecks => policy.supply_chain().semver().is_empty(),
            RequiredTool::FullMutants => policy.mutation().due(Some(FullRun::Weekly)).is_none(),
            RequiredTool::TestCpuTimeBudget => policy.budgets().whole(&Measure::TestCpuTime).is_none(),
            _ => false,
        }
        .then_some(Gap::TurnedOff(self))
    }

    pub fn fix(self) -> &'static str {
        match self {
            RequiredTool::Ci => {
                "add a step `run: cargo xtask ci` (checkout with fetch-depth: 0) to a workflow on pull_request"
            }
            RequiredTool::CargoDeny => {
                "install it before `cargo xtask ci`: `uses: taiki-e/install-action@v2` with `tool: cargo-deny@0.20.2`"
            }
            RequiredTool::CargoSemverChecks => {
                "install it before `cargo xtask ci` (`uses: taiki-e/install-action@v2` with `tool: cargo-semver-checks@0.51.0`) and list your library crates in `[supply_chain] semver`"
            }
            RequiredTool::DiffMutants => {
                "add a step `run: cargo xtask mutants` (cargo-mutants@27.1.0 installed, checkout with fetch-depth: 0) to a workflow on pull_request"
            }
            RequiredTool::FullMutants => {
                "add a step `run: cargo xtask mutants-full` (cargo-mutants@27.1.0 installed) to a workflow on push or schedule, and list \"weekly\" in `[mutation] full_run`"
            }
            RequiredTool::Stacked => {
                "add a step `run: cargo xtask stacked` with env GH_TOKEN and PULL_REQUEST (permissions: pull-requests: read) to a workflow on pull_request"
            }
            RequiredTool::DependabotUpdates => {
                "list this workspace in .github/dependabot.yml: an `updates` entry with `package-ecosystem: cargo`, `directory: /<workspace path>`, `open-pull-requests-limit: 0` (alerts on, version-update pull requests off) and `schedule: interval: weekly`"
            }
            RequiredTool::DependabotAlerts => {
                "turn on Dependabot alerts, store a fine-grained token with repository permission \"Dependabot alerts: read\" as secret DEPENDABOT_TOKEN, pass it as env `GH_TOKEN: ${{ secrets.DEPENDABOT_TOKEN }}` to the step that runs `cargo xtask ci` (github.token cannot read alerts), and fix or dismiss every open alert"
            }
            RequiredTool::TestCpuTimeBudget => {
                "copy this default into tooling/policy.toml:\n[budgets.test-cpu-time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_cpu_time\"\nallowance = { duration = 4000 }\nwindow = \"per_change\"\nsource = \"ci\""
            }
        }
    }
}
