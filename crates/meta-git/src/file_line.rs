use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileLine(u32);

impl From<u32> for FileLine {
    fn from(line: u32) -> FileLine {
        FileLine(line)
    }
}

impl fmt::Display for FileLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
