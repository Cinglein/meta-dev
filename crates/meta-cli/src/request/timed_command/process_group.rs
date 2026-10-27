use std::fmt;
use std::process::{Child, Command};

use meta_trace::SpawnSpan;

#[derive(Clone, Copy, Debug)]
pub struct ProcessGroup(u32);

impl ProcessGroup {
    const KILL: &'static str = "kill";
    const SIGNAL: &'static str = "-KILL";
    const END_OF_OPTIONS: &'static str = "--";

    pub fn led_by(child: &Child) -> Self {
        Self(child.id())
    }

    pub fn kill(self) {
        let mut kill = Command::new(Self::KILL);
        SpawnSpan::status_of(kill.args([Self::SIGNAL, Self::END_OF_OPTIONS, &self.to_string()])).ok();
    }
}

impl fmt::Display for ProcessGroup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "-{}", self.0)
    }
}
