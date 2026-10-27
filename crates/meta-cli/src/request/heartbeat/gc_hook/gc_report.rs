use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GcReport(String);

impl GcReport {
    pub fn shown(shown: impl fmt::Display) -> Self {
        let Ok(report) = format!("{shown}").parse();
        report
    }
}

impl FromStr for GcReport {
    type Err = std::convert::Infallible;

    fn from_str(report: &str) -> Result<Self, Self::Err> {
        Ok(Self(report.to_owned()))
    }
}
