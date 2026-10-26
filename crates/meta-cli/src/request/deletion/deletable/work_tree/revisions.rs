use std::str::FromStr;

use crate::request::deletion::kept::Kept;

#[derive(Debug)]
pub struct Revisions;

impl FromStr for Revisions {
    type Err = Kept;

    fn from_str(reply: &str) -> Result<Self, Self::Err> {
        reply.trim().is_empty().then_some(Self).ok_or(Kept::Unpushed)
    }
}
