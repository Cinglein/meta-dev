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
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unused = (&self.task, &self.rules, self.options.actor(), self.options.repository(), self.options.scratch());
        drop(unused);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
