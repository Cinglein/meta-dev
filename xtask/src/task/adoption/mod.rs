mod gap;
mod gaps;
mod required_tool;
mod scalar;
mod skip;
mod step_run;
mod trigger;
mod workflow;
mod working_directory;
mod yaml_node;

pub use gaps::Gaps;

use strum::IntoEnumIterator;
use task::{CommandLine, Policy, Program, Site, Stdout, Workspace};

use crate::task::Task;
use crate::task::failure::Failure;
use gap::Gap;
use required_tool::RequiredTool;
use step_run::StepRun;
use workflow::Workflow;
use working_directory::WorkingDirectory;

#[derive(Debug)]
pub struct Adoption {
    cargo: CommandLine,
    workflows: Vec<Workflow>,
    directory: WorkingDirectory,
}

impl Adoption {
    const GIT: &str = ".git";
    const VERSION: &str = "--version";

    pub fn checked(workspace: &Workspace, policy: &Policy) -> Result<(), Failure> {
        let root = workspace.root();
        let repository = root.ancestors().find(|directory| directory.join(Self::GIT).exists()).unwrap_or(root);
        let directory = repository.join(Workflow::DIRECTORY);
        let workflows = Workflow::all(&directory)
            .map_err(|error| Failure::from(task::Failure::Io { site: Site::File(directory), error }))?;
        let adoption = Adoption {
            cargo: workspace.line(Program::Cargo),
            workflows,
            directory: WorkingDirectory::within(repository, root),
        };
        let gaps = adoption.gaps(policy);
        println!("adoption: {} required tools checked", RequiredTool::iter().count());
        if gaps.is_empty() { Ok(()) } else { Err(Failure::Adoption(gaps)) }
    }

    fn gaps(&self, policy: &Policy) -> Gaps {
        let installed = |subcommand: &&str| {
            self.cargo.clone().argument(subcommand).argument(Self::VERSION).captured::<Stdout>().is_ok()
        };
        Gaps::new(
            RequiredTool::iter()
                .flat_map(|tool| {
                    let uninstalled = tool.subcommand().filter(|subcommand| !installed(subcommand));
                    let unrun = tool.task().and_then(|task| self.unrun(tool, task));
                    let gaps = uninstalled.map(|_| Gap::Uninstalled(tool)).into_iter().chain(unrun);
                    gaps.chain(tool.turned_off(policy))
                })
                .collect(),
        )
    }

    fn unrun(&self, tool: RequiredTool, task: Task) -> Option<Gap> {
        let runs =
            self.workflows.iter().flat_map(|workflow| workflow.runs(task.into(), &self.directory, tool.trigger()));
        let runs: Vec<StepRun> = runs.collect();
        match runs.iter().find(|run| run.skip().is_none()) {
            Some(_) => None,
            None => Some(runs.first().map_or_else(|| Gap::Absent(tool, self.directory.clone()), |run| run.gap(tool))),
        }
    }
}

#[cfg(test)]
mod tests {
    use meta_golden::Golden;
    use settings::EmbeddedDefaults;
    use task::{CommandLine, Policy, Program};

    use strum::IntoEnumIterator;

    use super::{Adoption, Gaps, RequiredTool, Workflow, WorkingDirectory};

    const CONSUMER: &str = include_str!("../../../../examples/consumer/tooling/policy.toml");
    const DROPPED: [&str; 3] = [
        "[budgets.test-cpu-time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_cpu_time\"\nallowance = { duration = 4000 }\nwindow = \"per_change\"\nsource = \"ci\"\n",
        "\"greeting\"",
        "\"every_main_push\", \"weekly\", \"on_demand\"",
    ];
    const FAILING: &str = "false";
    const FILE: &str = "ci.yml";
    const WORKFLOW: &str = "on:
  - pull_request
jobs:
  plan:
    steps:
      - run: cargo xtask mutants-diff-plan
  mutants:
    if: ${{ needs.plan.outputs.shards != '[]' }}
    continue-on-error: false
    steps:
      - &anchor
        run: |
          cargo xtask mutants | tee log
        shell: bash
  stacked:
    if: github.event_name == 'pull_request'
    steps:
      - run: cargo xtask stacked
  ci:
    steps:
      - name: whole
        run: |
          cargo xtask ci ${{ a || b }} || true
      - run: cargo xtask ci | tee summary
        working-directory: ./other/
  full:
    steps:
      - run: echo

# full runs belong on push or schedule
      - run: cargo xtask mutants-full
";
    const LINT_FILE: &str = "lint.yml";
    const LINT: &str = "on: pull_request
jobs:
  lint:
    steps:
      - run: cargo xtask stacked
        working-directory: other
      - run: cargo xtask mutants
        working-directory: other
        continue-on-error: true
";
    const MAIN_FILE: &str = "main.yml";
    const MAIN: &str = "on: [push, 'schedule']
jobs:
  full:
    steps:
      - run: cargo xtask mutants-full
        working-directory: other
";
    const DIRECTORIES: [&str; 2] = [".", "other"];

    #[test]
    fn missing_tools_steps_that_can_skip_and_absent_steps_are_gaps_with_their_fix() {
        let gaps = DIRECTORIES.map(|directory| {
            let Ok(directory) = directory.parse::<WorkingDirectory>();
            let workflows = [(FILE, WORKFLOW), (LINT_FILE, LINT), (MAIN_FILE, MAIN)].map(|(file, text)| {
                let Ok(root) = text.parse();
                Workflow::new(file.into(), root)
            });
            let cargo = CommandLine::new(Program::Executable(FAILING.into()));
            Adoption { cargo, workflows: workflows.into(), directory }.gaps(&Policy::defaults())
        });
        assert!(Gaps::new(Vec::new()).is_empty() && !gaps.iter().any(Gaps::is_empty));
        Golden::for_test(module_path!(), "gaps").checked(&gaps.map(|gaps| gaps.to_string()).join("\n")).unwrap();
    }

    #[test]
    fn a_policy_without_semver_crates_a_weekly_full_run_or_a_required_budget_fails_with_the_default_to_copy() {
        let stripped = DROPPED.iter().fold(CONSUMER.to_owned(), |text, dropped| text.replace(dropped, ""));
        let policy: Policy = toml::from_str(&stripped).unwrap();
        let tools = RequiredTool::iter().filter_map(|tool| tool.turned_off(&policy)).map(|gap| gap.to_string());
        Golden::for_test(module_path!(), "turned_off").checked(&tools.collect::<Vec<_>>().join("\n")).unwrap();
    }
}
