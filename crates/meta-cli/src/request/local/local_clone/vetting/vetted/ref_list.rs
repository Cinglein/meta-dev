use std::ffi::OsStr;
use std::str::FromStr;

use super::ref_line::RefLine;
use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefList(Vec<RefLine>);

impl RefList {
    pub fn remote(listed: Result<String, Kept>) -> Result<Self, Kept> {
        listed.map_err(|kept| kept.recast(Kept::Unreachable))?.parse()
    }

    pub fn headed(self, head: RefLine) -> Self {
        Self([self.0, vec![head]].concat())
    }

    pub fn named(&self, name: &str) -> Option<&RefLine> {
        self.0.iter().find_map(|line| line.named(name))
    }

    pub fn objects(&self) -> impl Iterator<Item = &OsStr> {
        self.0.iter().map(RefLine::object)
    }

    pub fn matched(&self, remote: &Self) -> Result<(), Kept> {
        (!self.0.iter().any(|local| remote.0.iter().any(|seen| local.diverges_from(seen))))
            .then_some(())
            .ok_or(Kept::Diverged)
    }
}

impl FromStr for RefList {
    type Err = Kept;

    fn from_str(listed: &str) -> Result<Self, Self::Err> {
        let lines = listed.lines().filter(|line| !line.trim().is_empty()).map(str::parse);
        lines.collect::<Result<_, ()>>().map(Self).map_err(|()| Kept::Unreadable)
    }
}
