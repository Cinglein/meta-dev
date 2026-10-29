use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;

use super::link::{Link, Scope};
use crate::request::local::local_clone::vetting::halt::Halt;
use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::look::Look;
use crate::request::local::local_clone::vetting::replay::Replay;
use crate::request::local::local_clone::vetting::seen::Seen;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkouts(Vec<String>);

impl Checkouts {
    const ON_BRANCH: [&'static str; 3] = ["symbolic-ref", "-q", "HEAD"];
    pub const LIST: [&'static str; 3] = ["worktree", "list", "--porcelain"];
    const BLOCKS: &'static str = "\n\n";
    const WORKTREE: &'static str = "worktree ";
    const BRANCH: &'static str = "branch ";
    const SEPARATOR: &'static str = "\0";

    pub fn held_now(link: &Link, branch: &str, replay: &Replay<'_>) -> Result<(), Halt> {
        let looks = [(Scope::Worktree, Self::ON_BRANCH.as_slice()), (Scope::Common, Self::LIST.as_slice())]
            .map(|(scope, words)| Look::Git(link.call(scope, &words.iter().map(OsStr::new).collect::<Vec<_>>())));
        let mut seen = replay.looked(looks.to_vec())?.into_iter();
        let mut next = || seen.next().ok_or_else(Replay::diverged).and_then(Seen::git);
        let head = next()?.ok();
        head.filter(|head| head.lines().next() == Some(branch)).ok_or(Kept::OffBranch)?;
        let Ok(checkouts) = next()??.parse::<Self>();
        Ok(checkouts.unshared(branch, Self::entry(branch, link.path()).as_ref())?)
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
