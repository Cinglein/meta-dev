mod step;
mod timing;

use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::failure::Failure;
use step::Step;
use timing::Timing;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiConfig {
    step: Vec<Step>,
}

impl CiConfig {
    const TOTAL: &str = "total";

    pub fn run_in(&self, root: &Path) -> Result<(), Failure> {
        let elapsed = self
            .step
            .iter()
            .map(|step| step.run_in(root).map(|timing| timing.elapsed()))
            .sum::<Result<Duration, Failure>>();
        println!("{}", Timing::new(Self::TOTAL, elapsed?));
        Ok(())
    }
}
