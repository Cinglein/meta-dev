use std::fmt;

use meta_schema::{Id, Task};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenTasks(Vec<Id<Task>>);

impl From<Vec<Id<Task>>> for OpenTasks {
    fn from(tasks: Vec<Id<Task>>) -> Self {
        Self(tasks)
    }
}

impl fmt::Display for OpenTasks {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.split_first() {
            None => formatter.write_str("none is open; open one with --instruments"),
            Some((first, rest)) => {
                write!(formatter, "{first}")?;
                rest.iter().try_for_each(|task| write!(formatter, ", {task}"))
            }
        }
    }
}
