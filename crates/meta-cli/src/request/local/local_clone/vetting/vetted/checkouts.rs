use std::path::Path;
use std::str::FromStr;

use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkouts(Vec<String>);

impl Checkouts {
    const BLOCKS: &'static str = "\n\n";
    const WORKTREE: &'static str = "worktree ";
    const BRANCH: &'static str = "branch ";
    const SEPARATOR: &'static str = "\0";

    pub fn unshared(&self, branch: &str, path: &Path) -> Result<(), Kept> {
        let ours = path.to_str().map(|path| [path, Self::SEPARATOR, branch].concat());
        let held = |entry: &String| entry.ends_with(&[Self::SEPARATOR, branch].concat());
        (!self.0.iter().any(|entry| held(entry) && Some(entry) != ours.as_ref())).then_some(()).ok_or(Kept::Shared)
    }
}

impl FromStr for Checkouts {
    type Err = std::convert::Infallible;

    fn from_str(listed: &str) -> Result<Self, Self::Err> {
        let held = listed.split(Self::BLOCKS).filter_map(|block| {
            let path = block.lines().find_map(|line| line.strip_prefix(Self::WORKTREE))?;
            let branch = block.lines().find_map(|line| line.strip_prefix(Self::BRANCH))?;
            Some([path, Self::SEPARATOR, branch].concat())
        });
        Ok(Self(held.collect()))
    }
}
