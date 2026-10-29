use super::checkouts::Checkouts;
use super::link::Link;
use super::ref_list::RefList;
use crate::request::local::local_clone::vetting::context::Context;
use crate::request::local::local_clone::vetting::halt::Halt;
use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::replay::Replay;

// Fresh reads the worktree list and the remote now; Swept uses the ones the sweep read once.
#[derive(Clone, Copy, Debug)]
pub enum Survey<'sweep> {
    Fresh,
    Swept { checkouts: &'sweep Result<Checkouts, Kept>, remote: &'sweep Result<RefList, Kept> },
}

impl Survey<'_> {
    pub fn held(self, link: &Link, branch: &str, replay: &Replay<'_>) -> Result<(), Halt> {
        match self {
            Self::Fresh => Checkouts::held_now(link, branch, replay),
            Self::Swept { checkouts, .. } => Ok(checkouts.as_ref().map_err(Kept::clone)?.held(branch, link.path())?),
        }
    }

    pub fn remote_refs(self, context: &Context, replay: &Replay<'_>) -> Result<RefList, Halt> {
        match self {
            Self::Fresh => Ok(RefList::remote(replay.git(context.remote_refs())?)?),
            Self::Swept { remote, .. } => Ok(remote.clone()?),
        }
    }
}
