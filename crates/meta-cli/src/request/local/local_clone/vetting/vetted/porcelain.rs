use std::path::Path;
use std::str::FromStr;

mod change;

use crate::request::local::local_clone::vetting::kept::Kept;
use change::Change;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Porcelain(Vec<Change>);

impl Porcelain {
    const SEPARATOR: char = '\0';

    pub fn ignored(&self) -> Result<Vec<&Path>, Kept> {
        self.0.iter().find_map(Change::kept).map_or(Ok(()), Err)?;
        Ok(self.0.iter().filter_map(Change::ignored).collect())
    }
}

impl FromStr for Porcelain {
    type Err = std::convert::Infallible;

    fn from_str(listed: &str) -> Result<Self, Self::Err> {
        let changes =
            listed.split(Self::SEPARATOR).filter(|entry| !entry.is_empty()).filter_map(|entry| entry.parse().ok());
        Ok(Self(changes.collect()))
    }
}
