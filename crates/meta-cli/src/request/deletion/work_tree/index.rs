use std::str::FromStr;

use crate::request::deletion::kept::Kept;

#[derive(Debug)]
pub struct Index;

impl Index {
    const SEPARATOR: char = '\0';
    const CACHED: &'static str = "H";
    const GITLINK: &'static str = "160000";

    fn vetted(entry: &str) -> Result<(), Kept> {
        match entry.split_whitespace().collect::<Vec<_>>().as_slice() {
            [Self::CACHED, Self::GITLINK, ..] => Err(Kept::Submodules),
            [Self::CACHED, _, ..] => Ok(()),
            _ => Err(Kept::IndexFlags),
        }
    }
}

impl FromStr for Index {
    type Err = Kept;

    fn from_str(reply: &str) -> Result<Self, Self::Err> {
        reply.split(Self::SEPARATOR).filter(|entry| !entry.is_empty()).try_for_each(Self::vetted).map(|()| Self)
    }
}
