use super::checkouts::Checkouts;
use super::link::Link;
use super::ref_list::RefList;
use crate::request::local::local_clone::vetting::context::Context;
use crate::request::local::local_clone::vetting::inspection::Inspection;
use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Copy, Debug)]
pub enum Survey<'sweep> {
    Fresh,
    Swept(&'sweep Result<Checkouts, Kept>),
}

impl Survey<'_> {
    pub fn held(self, link: &Link, branch: &str, host: &impl Inspection) -> Result<(), Kept> {
        match self {
            Self::Fresh => Checkouts::held_now(link, branch, host),
            Self::Swept(checkouts) => checkouts.as_ref().map_err(Kept::clone)?.held(branch, link.path()),
        }
    }

    pub fn remote_refs(self, context: &Context, host: &impl Inspection) -> Result<RefList, Kept> {
        match self {
            Self::Fresh => context.remote_refs_now(host),
            Self::Swept(_) => context.remote_refs(host).clone(),
        }
    }
}
