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

    pub(super) fn unsound(&self) -> Option<CheckName> {
        (self.conclusion != Conclusion::Success).then(|| self.name.clone())
    }

    pub(super) fn is_named(&self, name: &str) -> bool {
        self.name == CheckName::from(name)
    }
}

impl FromStr for StepOutcome {
    type Err = Failure;

    fn from_str(listed: &str) -> Result<StepOutcome, Failure> {
        let unreadable = || Failure::Usage(format!("{listed} is not step=outcome"));
        let (name, conclusion) = listed.split_once('=').ok_or_else(unreadable)?;
        Ok(StepOutcome { name: name.into(), conclusion: conclusion.parse().unwrap_or(Conclusion::Unknown) })
    }
}
