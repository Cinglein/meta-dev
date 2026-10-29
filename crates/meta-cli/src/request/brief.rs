mod brief_request;
mod options;
mod rules;

use std::fmt;

use meta_schema::NewTask;

pub use brief_request::BriefRequest;
pub use options::BriefOptions;
pub use rules::Rules;

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
        let (id, actor) = (self.task.task(), self.options.actor());
        let (repository, scratch) = (self.options.repository(), self.options.scratch());
        writeln!(formatter, "brief for {id} as {actor}")?;
        writeln!(formatter, "task: {}{}", self.task.text(), self.task.placement())?;
        writeln!(formatter, "setup, in order:")?;
        writeln!(formatter, "1. gh repo clone {repository} {scratch}/{id}")?;
        writeln!(formatter, "2. in the clone: nice -n 19 env CARGO_BUILD_JOBS=2 cargo build --release -p meta-cli")?;
        writeln!(formatter, "3. in the clone: META_ACTOR={actor} target.noindex/release/meta claim {id} --worktree")?;
        writeln!(formatter, "4. work in the worktree path it prints")?;
        writeln!(formatter, "rules ({}):", Rules::FILE)?;
        writeln!(formatter, "{}", self.rules)
    }
}

#[cfg(test)]
mod tests;
