use std::env;

use crate::task::failure::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Runner {
    Ci,
    Local,
}

impl Runner {
    const CI: &str = "CI";

    pub fn ambient() -> Runner {
        env::var_os(Self::CI).map_or(Runner::Local, |_| Runner::Ci)
    }

    pub fn unreached(self, failure: Failure, skipped: &str) -> Result<(), Failure> {
        match self {
            Runner::Ci => Err(Failure::Unreached { skipped: skipped.to_owned(), failure: Box::new(failure) }),
            Runner::Local => {
                eprintln!("{skipped}: unchecked locally, blocks in CI: {failure}");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Runner;
    use crate::task::failure::Failure;

    const SKIPPED: &str = "skipped";
    const PASSED: [(Runner, bool); 2] = [(Runner::Ci, false), (Runner::Local, true)];

    #[test]
    fn an_unreached_check_blocks_in_ci_and_is_only_reported_locally() {
        let passed =
            |(runner, _): (Runner, bool)| runner.unreached(Failure::from(task::Failure::NoWorkspace), SKIPPED).is_ok();
        assert_eq!(PASSED.map(passed), PASSED.map(|(_, passed)| passed));
    }
}
