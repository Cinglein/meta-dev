mod job_guard;
mod shell_line;

use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

use crate::task::adoption::skip::Skip;
use crate::task::adoption::step_run::StepRun;
use crate::task::adoption::trigger::Trigger;
use crate::task::adoption::working_directory::WorkingDirectory;
use crate::task::adoption::yaml_node::YamlNode;
use job_guard::JobGuard;
use shell_line::ShellLine;

#[derive(Debug)]
pub struct Workflow {
    file: PathBuf,
    root: YamlNode,
}

impl Workflow {
    pub const DIRECTORY: &str = ".github/workflows";
    const EXTENSIONS: [&str; 2] = ["yml", "yaml"];
    const ON: &str = "on";
    const JOBS: &str = "jobs";
    const STEPS: &str = "steps";
    const RUN: &str = "run";
    const CONDITION: &str = "if";
    const SHELL: &str = "shell";
    const PIPEFAIL_SHELL: &str = "bash";
    const WORKING_DIRECTORY: &str = "working-directory";

    #[cfg(test)]
    pub fn new(file: PathBuf, root: YamlNode) -> Workflow {
        Workflow { file, root }
    }

    pub fn all(directory: &Path) -> io::Result<Vec<Workflow>> {
        let listed = fs::read_dir(directory).map(Iterator::collect::<io::Result<Vec<DirEntry>>>);
        let entries = match listed {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            listed => listed.and_then(|entries| entries),
        }?;
        let mut paths: Vec<PathBuf> = entries
            .iter()
            .map(DirEntry::path)
            .filter(|path| {
                path.extension().and_then(OsStr::to_str).is_some_and(|known| Self::EXTENSIONS.contains(&known))
            })
            .collect();
        paths.sort();
        paths.iter().map(|path| Self::read(path)).collect()
    }

    pub fn read(path: &Path) -> io::Result<Workflow> {
        let Ok(root) = fs::read_to_string(path)?.parse::<YamlNode>();
        Ok(Workflow { file: Path::new(Self::DIRECTORY).join(path.file_name().unwrap_or_default()), root })
    }

    pub fn runs(&self, task: &str, directory: &WorkingDirectory, trigger: Trigger) -> Vec<StepRun> {
        let on = self.root.child(Self::ON).filter(|on| on.names().any(|name| trigger.events().contains(&name)));
        let untriggered = on.map_or(Some(Skip::Untriggered(trigger)), |_| None);
        self.jobs()
            .iter()
            .flat_map(|job| job.items(Self::STEPS).iter().map(move |step| (job, step)))
            .filter(|(_, step)| {
                step.scalar_of(Self::WORKING_DIRECTORY).parse::<WorkingDirectory>().is_ok_and(|at| at == *directory)
            })
            .filter_map(|(job, step)| {
                let call = Self::call(step, task)?;
                let skip = untriggered.clone().or_else(|| self.job_skip(job));
                let skip = skip.or_else(|| Self::step_skip(step, call.skip()));
                Some(StepRun::new(format!("{self} job {}", job.name()), skip))
            })
            .collect()
    }

    fn jobs(&self) -> &[YamlNode] {
        self.root.items(Self::JOBS)
    }

    fn job_skip(&self, job: &YamlNode) -> Option<Skip> {
        let planned = |condition: &&YamlNode| {
            let Ok(guard) = condition.scalar().parse::<JobGuard>();
            let planner =
                guard.planner().zip(self.root.child(Self::JOBS)).and_then(|(planner, jobs)| jobs.child(planner));
            planner.is_some_and(|planner| planner.child(Self::CONDITION).is_none())
        };
        let condition = job.child(Self::CONDITION).filter(|condition| !planned(condition));
        condition.map(|condition| Skip::JobCondition(condition.scalar().clone())).or_else(|| Skip::tolerated(job))
    }

    fn step_skip(step: &YamlNode, call: Option<Skip>) -> Option<Skip> {
        let call = call.filter(|skip| *skip != Skip::Unpiped || step.scalar_of(Self::SHELL) != Self::PIPEFAIL_SHELL);
        let condition = step.child(Self::CONDITION).map(|condition| Skip::StepCondition(condition.scalar().clone()));
        condition.or_else(|| Skip::tolerated(step)).or(call)
    }

    fn call(step: &YamlNode, task: &str) -> Option<ShellLine> {
        let run = step.scalar_of(Self::RUN).lines();
        run.filter_map(|line| line.parse::<ShellLine>().ok()).find(|line| line.calls(task))
    }
}

impl fmt::Display for Workflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.file.display())
    }
}
