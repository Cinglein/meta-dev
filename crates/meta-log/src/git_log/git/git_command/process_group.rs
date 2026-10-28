use std::fmt;
use std::io;
use std::process::{Child, Command};

#[derive(Clone, Copy, Debug)]
pub struct ProcessGroup(u32);

impl ProcessGroup {
    const KILL: &'static str = "kill";
    const SIGNAL: &'static str = "-KILL";
    const END_OF_OPTIONS: &'static str = "--";

    pub fn led_by(child: &Child) -> Self {
        Self(child.id())
    }

    pub fn kill(self) -> io::Result<()> {
        Command::new(Self::KILL).args([Self::SIGNAL, Self::END_OF_OPTIONS, &self.to_string()]).status().map(drop)
    }
}

impl fmt::Display for ProcessGroup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "-{}", self.0)
    }
}
