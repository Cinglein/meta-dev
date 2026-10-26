mod task_line;

use std::fmt;
use std::iter::once;

use meta_state::TaskState;

use super::Status;
use task_line::TaskLine;

#[derive(Clone, Copy, Debug)]
pub struct Tree<'view> {
    status: Status<'view>,
}

impl<'view> Tree<'view> {
    const ROOT: &'static str = "root";
    const INDENT: &'static str = "  ";

    pub fn new(status: Status<'view>) -> Self {
        Self { status }
    }

    fn headers(task: &TaskState) -> Vec<String> {
        task.new_task().placement().division().map_or_else(
            || vec![Self::ROOT.to_owned()],
            |division| division.lineage().iter().map(ToString::to_string).collect(),
        )
    }

    fn fresh(lineage: &[String], before: Option<&Vec<String>>) -> Vec<String> {
        let shown = |depth: usize| before.and_then(|before| before.get(depth));
        let indented = |(depth, header): (usize, &String)| format!("{}{header}", Self::INDENT.repeat(depth));
        lineage.iter().enumerate().filter(|(depth, header)| shown(*depth) != Some(*header)).map(indented).collect()
    }
}

impl fmt::Display for Tree<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tasks: Vec<&TaskState> = self.status.shown().collect();
        tasks.sort_by_key(|task| task.new_task().placement().division().cloned());
        let lineages: Vec<Vec<String>> = tasks.iter().map(|task| Self::headers(task)).collect();
        let previous = once(None).chain(lineages.iter().map(Some));
        tasks.iter().zip(&lineages).zip(previous).try_for_each(|((task, lineage), before)| {
            Self::fresh(lineage, before).iter().try_for_each(|header| writeln!(formatter, "{header}"))?;
            let line = TaskLine::new(self.status, task);
            writeln!(formatter, "{}{line}", Self::INDENT.repeat(lineage.len()))
        })
    }
}
