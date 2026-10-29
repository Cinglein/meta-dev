use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::query::Query;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;

#[derive(Clone, Copy, Debug)]
pub struct PausedRoots;

impl Query for PausedRoots {
    type View = ();
    const ARGV: VetArgv = VetArgv::PausedRoots;

    fn viewed(&self, out: &str) -> Result<(), Kept> {
        out.lines().next().is_none().then_some(()).ok_or(Kept::Paused)
    }
}
