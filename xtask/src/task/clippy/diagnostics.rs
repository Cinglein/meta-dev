use task::FailureSet;

#[derive(Debug)]
pub(super) struct Diagnostics;

impl Diagnostics {
    pub(super) fn parse(_printed: &str) -> Diagnostics {
        Diagnostics
    }

    pub(super) fn found(&self) -> FailureSet {
        FailureSet::default()
    }

    pub(super) fn rendered(&self) -> String {
        String::new()
    }
}
