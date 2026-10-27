use serde::Deserialize;
use task::BaselineFile;

#[derive(Debug, Default, Deserialize)]
pub struct PriorAllowance {
    baseline: Option<BaselineFile>,
}

impl PriorAllowance {
    pub fn baseline(&self) -> Option<&BaselineFile> {
        self.baseline.as_ref()
    }
}
