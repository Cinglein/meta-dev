use std::fmt;

use meta_state::TaskState;

use crate::status::Status;
use crate::status::effort_spend::EffortSpend;

#[derive(Clone, Copy, Debug)]
pub struct TaskLine<'view> {
    status: Status<'view>,
    task: &'view TaskState,
}

impl<'view> TaskLine<'view> {
    const SEPARATOR: &'static str = " | ";
    const SHOWN_WORDS_LEN: usize = 8;

    pub fn new(status: Status<'view>, task: &'view TaskState) -> Self {
        Self { status, task }
    }
}

impl fmt::Display for TaskLine<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let planned = self.task.new_task();
        let id = planned.task();
        write!(formatter, "{id} {}", self.status.state.status_of(self.task))?;
        self.status.pulls(self.task).iter().try_for_each(|pull_request| {
            write!(formatter, "{}{pull_request} {}", Self::SEPARATOR, self.status.github.sight(*pull_request))
        })?;
        let holders: Vec<String> = self.status.state.holders(id).iter().map(ToString::to_string).collect();
        let held =
            holders.first().map_or_else(|| "no holder".to_owned(), |_| format!("held by {}", holders.join(", ")));
        write!(formatter, "{}{held}", Self::SEPARATOR)?;
        planned.effort_allowance().map_or(Ok(()), |effort| {
            write!(formatter, "{}effort {effort}, {}", Self::SEPARATOR, EffortSpend::measured_for(self.task))
        })?;
        planned.due_time().map_or(Ok(()), |due| write!(formatter, "{}due {due}", Self::SEPARATOR))?;
        let text = planned.text().to_string();
        let words: Vec<&str> = text.split_whitespace().collect();
        let kept = words.get(..Self::SHOWN_WORDS_LEN).unwrap_or(&words).join(" ");
        let more = words.get(Self::SHOWN_WORDS_LEN).map_or("", |_| "…");
        write!(formatter, "{}{kept}{more}", Self::SEPARATOR)
    }
}
