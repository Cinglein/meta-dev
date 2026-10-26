use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct ProcessId(u32);

impl ProcessId {
    pub fn current() -> Self {
        Self(std::process::id())
    }
}

impl fmt::Display for ProcessId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
