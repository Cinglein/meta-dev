use std::fmt;

use crate::report::Report;

#[derive(Debug)]
pub enum Printed {
    Report(Report),
    Text(String),
    Under { command: String, text: Box<Printed> },
}

impl fmt::Display for Printed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Printed::Report(report) => write!(formatter, "{report}"),
            Printed::Text(text) => formatter.write_str(text),
            Printed::Under { command, text } => write!(formatter, "{command} failed:\n{text}"),
        }
    }
}
