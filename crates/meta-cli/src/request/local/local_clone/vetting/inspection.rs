use std::path::{Path, PathBuf};

use super::alternates::Alternates;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::git_call::GitCall;
use super::kept::Kept;
use super::occupancy::Occupancy;
use super::thaw::Thaw;
use crate::request::local::local_clone::quarantine::Stamp;

pub trait Inspection {
    fn entry(&self, path: &Path) -> EntryKind;
    fn canonical(&self, path: &Path) -> Option<PathBuf>;
    fn text(&self, path: &Path) -> Option<String>;
    fn git(&self, call: &GitCall) -> Result<String, Kept>;
    fn walked(&self, path: &Path) -> Vec<Found>;
    fn neighbours(&self, root: &Path) -> Vec<Alternates>;
    fn occupancy(&self, paths: &[PathBuf]) -> Occupancy;
    fn stamp(&self) -> Stamp;
    fn frozen(&self, quarantine: &Path, plan: Thaw) -> Result<Thaw, Kept>;
    fn thawed(&self, thaw: &Thaw);
}
