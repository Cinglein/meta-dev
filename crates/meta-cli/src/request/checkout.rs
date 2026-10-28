use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use std::time::Instant;

use crate::request::ci::Commit;
use crate::request::project::BranchName;
use crate::request::timed_command::TimedCommand;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checkout {
    head: Option<Commit>,
    branch: Option<BranchName>,
}

impl Checkout {
    const HEADS: &'static str = "refs/heads/";
    const NAMED_HEAD: [&'static str; 4] = ["rev-parse", "HEAD", "--symbolic-full-name", "HEAD"];
    const DETACHED: &'static str = "HEAD";
    const ATTACHED: &'static str = "HEAD -> ";
    const DECORATIONS: &'static str = ", ";

    pub fn read(root: &Path, deadline: Instant) -> Self {
        let mut command = Command::new("git");
        command.arg("-C").arg(root).args(Self::NAMED_HEAD);
        let Ok(checkout) = TimedCommand::new(command, deadline).stdout().unwrap_or_default().parse();
        checkout
    }

    pub fn decorated(head: &Commit, decorations: &str) -> Option<Self> {
        let named = decorations.split(Self::DECORATIONS).find_map(|decoration| {
            (decoration == Self::DETACHED).then_some(decoration).or_else(|| decoration.strip_prefix(Self::ATTACHED))
        })?;
        Some(Self { head: Some(head.clone()), branch: Self::branch_named(named) })
    }

    fn branch_named(named: &str) -> Option<BranchName> {
        named.trim().strip_prefix(Self::HEADS).and_then(|name| name.parse().ok())
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
        Ok(Self { head, branch: Self::branch_named(named) })
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
