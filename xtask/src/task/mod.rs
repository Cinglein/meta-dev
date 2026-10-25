mod failure;
mod project;

use std::str::FromStr;

use crate::task::failure::Failure;
use project::Project;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Task {
    Lint,
    Baseline,
    Ci,
    Graph,
    Usage,
}

impl Task {
    const USAGE: &str = "usage: cargo xtask <lint|baseline|ci|graph>";

    pub fn run(self) -> Result<(), Failure> {
        let project = Project::locate()?;
        match self {
            Task::Lint => Ok(project.lints().lint(&project.prior()?).map(|tally| print!("{tally}"))?),
            Task::Baseline => Ok(project.lints().shrink_baseline().map(|tally| print!("{tally}"))?),
            Task::Ci => {
                let outcome = project.run_ci();
                if let Err(failure) = project.changed_surfaces().and_then(|changed| Ok(changed.announce()?)) {
                    eprintln!("surfaces: {failure}");
                }
                Ok(outcome?)
            }
            Task::Graph => Ok(project.lints().crate_graphs().map(|report| print!("{report}"))?),
            Task::Usage => Err(Failure::Usage(Self::USAGE)),
        }
    }
}

impl FromStr for Task {
    type Err = Failure;

    fn from_str(text: &str) -> Result<Task, Failure> {
        match text {
            "lint" => Ok(Task::Lint),
            "baseline" => Ok(Task::Baseline),
            "ci" => Ok(Task::Ci),
            "graph" => Ok(Task::Graph),
            _ => Err(Failure::Usage(Self::USAGE)),
        }
    }
}
