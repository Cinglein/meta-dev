use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;

use super::link::{Link, Scope};
use crate::request::local::local_clone::vetting::context::Context;
use crate::request::local::local_clone::vetting::git_call::GitCall;
use crate::request::local::local_clone::vetting::inspection::Inspection;
use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkouts(Vec<String>);

impl Checkouts {
    const ON_BRANCH: [&'static str; 3] = ["symbolic-ref", "-q", "HEAD"];
    const LIST: [&'static str; 3] = ["worktree", "list", "--porcelain"];
    const BLOCKS: &'static str = "\n\n";
    const WORKTREE: &'static str = "worktree ";
    const BRANCH: &'static str = "branch ";
    const SEPARATOR: &'static str = "\0";

    pub fn listed(context: &Context, host: &impl Inspection) -> Result<Self, Kept> {
        let Ok(checkouts) = host.git(&GitCall::common(context.common(), &Self::LIST.map(OsStr::new)))?.parse();
        Ok(checkouts)
    }

    pub fn held_now(link: &Link, branch: &str, host: &impl Inspection) -> Result<(), Kept> {
        let head = link.git(host, Scope::Worktree, &Self::ON_BRANCH.map(OsStr::new)).ok();
        head.filter(|head| head.lines().next() == Some(branch)).ok_or(Kept::OffBranch)?;
        let Ok(checkouts) = link.git(host, Scope::Common, &Self::LIST.map(OsStr::new))?.parse::<Self>();
        checkouts.unshared(branch, Self::entry(branch, link.path()).as_ref())
    }

    pub fn held(&self, branch: &str, path: &Path) -> Result<(), Kept> {
        let ours = Self::entry(branch, path);
        ours.as_ref().filter(|ours| self.0.contains(ours)).ok_or(Kept::OffBranch)?;
        self.unshared(branch, ours.as_ref())
    }

    fn entry(branch: &str, path: &Path) -> Option<String> {
        path.to_str().map(|path| [path, Self::SEPARATOR, branch].concat())
    }

    fn unshared(&self, branch: &str, ours: Option<&String>) -> Result<(), Kept> {
        let held = |entry: &String| entry.ends_with(&[Self::SEPARATOR, branch].concat());
        (!self.0.iter().any(|entry| held(entry) && Some(entry) != ours)).then_some(()).ok_or(Kept::Shared)
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
