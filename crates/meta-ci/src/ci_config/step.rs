use std::path::Path;
use std::process::Command;
use std::time::Instant;

use serde::Deserialize;

use crate::ci_config::timing::Timing;
use crate::failure::Failure;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    name: String,
    program: String,
    #[serde(default)]
    args: Vec<String>,
}

impl Step {
    pub fn run_in(&self, root: &Path) -> Result<Timing, Failure> {
        let started = Instant::now();
        let status = Command::new(&self.program)
            .args(&self.args)
            .current_dir(root)
            .status()
            .map_err(|error| Failure::Spawn { step: self.name.clone(), error })?;
        let timing = Timing::new(&self.name, started.elapsed());
        println!("{timing}");
        status.success().then_some(timing).ok_or_else(|| Failure::Step(self.name.clone()))
    }
}
