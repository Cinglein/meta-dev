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
                eprintln!("{skipped}: not checked, the network failed (blocks in CI): {failure}");
                Ok(())
            }
        }
    }
}
