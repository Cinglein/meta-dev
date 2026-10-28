use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

use meta_run::Run;

use crate::request::ci::Commit;
use crate::request::project::BranchName;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checkout {
    head: Option<Commit>,
    branch: Option<BranchName>,
}

impl Checkout {
    const HEADS: &'static str = "refs/heads/";
    const NAMED_HEAD: [&'static str; 4] = ["rev-parse", "HEAD", "--symbolic-full-name", "HEAD"];

    pub fn read(root: &Path, deadline: Instant) -> Self {
        let named = Run::git(root).args(Self::NAMED_HEAD).stdout(deadline);
        let Ok(checkout) = named.unwrap_or_default().parse();
        checkout
    }

    pub fn head(&self) -> Option<&Commit> {
        self.head.as_ref()
    }

    pub fn branch(&self) -> Option<&BranchName> {
        self.branch.as_ref()
    }
}

impl FromStr for Checkout {
    type Err = std::convert::Infallible;

    fn from_str(parsed: &str) -> Result<Self, Self::Err> {
        let (head, named) = parsed.trim().split_once('\n').unwrap_or_default();
        let head = Some(head).filter(|head| !head.is_empty()).and_then(|head| head.parse().ok());
        let branch = named.trim().strip_prefix(Self::HEADS).and_then(|name| name.parse().ok());
        Ok(Self { head, branch })
    }
}

#[cfg(test)]
mod tests {
    use super::{BranchName, Checkout};

    const ON_BRANCH: &str = "0123456789abcdef0123456789abcdef01234567\nrefs/heads/task-00000000000000a1\n";
    const DETACHED: &str = "0123456789abcdef0123456789abcdef01234567\nHEAD\n";

    #[test]
    fn a_checkout_names_its_branch_only_when_head_is_on_one() {
        let [on, off] = [ON_BRANCH, DETACHED].map(|text| text.parse::<Checkout>().unwrap_or_default());
        assert!(on.head() == off.head() && on.head().is_some());
        assert!(on.branch().and_then(BranchName::task).is_some() && off.branch().is_none());
    }
}
