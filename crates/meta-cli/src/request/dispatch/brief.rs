use std::fmt;
use std::path::PathBuf;

use meta_schema::{Actor, NewTask};

use super::LeaseLine;

#[derive(Debug)]
pub struct Brief {
    agent: Actor,
    task: NewTask,
    project: Option<PathBuf>,
    lease: LeaseLine,
}

impl Brief {
    pub const PROJECT: &'static str = "project: ";

    pub fn new(agent: Actor, task: NewTask, project: Option<PathBuf>, lease: LeaseLine) -> Self {
        Self { agent, task, project, lease }
    }
}

impl fmt::Display for Brief {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "brief for {} as {}", self.task.task(), self.agent)?;
        writeln!(formatter, "task: {}", self.task)?;
        self.project.as_ref().map_or(Ok(()), |root| writeln!(formatter, "{}{}", Self::PROJECT, root.display()))?;
        writeln!(formatter, "{}", self.lease)
    }
}
