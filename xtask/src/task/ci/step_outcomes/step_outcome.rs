mod conclusion;

use std::str::FromStr;

use task::{CheckName, Failure};

use conclusion::Conclusion;

#[derive(Debug)]
pub(super) struct StepOutcome {
    name: CheckName,
    conclusion: Conclusion,
}

impl StepOutcome {
    pub(super) fn passing(&self) -> Option<&CheckName> {
        (self.conclusion == Conclusion::Success).then_some(&self.name)
    }

    pub(super) fn failing(&self) -> Option<CheckName> {
        matches!(self.conclusion, Conclusion::Failure | Conclusion::Cancelled).then(|| self.name.clone())
    }
}

impl FromStr for StepOutcome {
    type Err = Failure;

    fn from_str(listed: &str) -> Result<StepOutcome, Failure> {
        let unreadable = || Failure::Usage(format!("{listed} is not step=outcome"));
        let (name, conclusion) = listed.split_once('=').ok_or_else(unreadable)?;
        Ok(StepOutcome { name: name.into(), conclusion: conclusion.parse().map_err(|_| unreadable())? })
    }
}
