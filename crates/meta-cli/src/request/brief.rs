mod brief_failure;
mod brief_request;
mod options;
mod rules;

use std::fmt;

use meta_schema::NewTask;

pub use brief_failure::BriefFailure;
pub use brief_request::BriefRequest;
pub use options::BriefOptions;
pub use rules::Rules;

const REMOTE: &str = "origin";

#[derive(Debug)]
pub struct Brief {
    task: NewTask,
    rules: Rules,
    options: BriefOptions,
}

impl Brief {
    #[must_use]
    pub fn new(task: NewTask, rules: Rules, options: BriefOptions) -> Self {
        Self { task, rules, options }
    }
}

impl fmt::Display for Brief {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.task.task();
        let actor = self.options.acting_as();
        let repository = self.options.repository();
        let scratch = self.options.scratch();
        writeln!(formatter, "brief for {id} as {actor}")?;
        writeln!(formatter, "task: {}{}", self.task.text(), self.task.placement())?;
        let addresses: Vec<String> = self.task.addresses().iter().map(ToString::to_string).collect();
        (!addresses.is_empty()).then(|| writeln!(formatter, "addresses: {}", addresses.join(" "))).transpose()?;
        self.task.effort_allowance().map(|effort| writeln!(formatter, "effort: {effort}")).transpose()?;
        self.task.due_time().map(|due| writeln!(formatter, "due: {due}")).transpose()?;
        writeln!(formatter, "setup, in order:")?;
        writeln!(formatter, "1. gh repo clone {repository} {scratch}/{id}")?;
        writeln!(formatter, "2. in the clone: git fetch {REMOTE} {}", meta_log::Remote::BRANCH)?;
        writeln!(formatter, "3. in the clone: cargo build --release -p meta-cli (cargo rules below apply)")?;
        writeln!(formatter, "4. in the clone: META_ACTOR={actor} target.noindex/release/meta claim {id} --worktree")?;
        writeln!(formatter, "5. work in the worktree path it prints")?;
        writeln!(formatter, "rules ({}):", Rules::FILE)?;
        writeln!(formatter, "{}", self.rules)
    }
}

#[cfg(test)]
mod tests;
