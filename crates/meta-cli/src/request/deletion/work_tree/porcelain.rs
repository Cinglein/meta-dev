mod change;

use std::path::Path;
use std::str::FromStr;

use crate::request::deletion::kept::Kept;
use change::Change;

#[derive(Debug)]
pub struct Porcelain(Vec<Change>);

impl Porcelain {
    const SEPARATOR: char = '\0';

    pub fn settled(&self, root: &Path) -> Result<(), Kept> {
        self.0.iter().find_map(|change| change.kept(root)).map_or(Ok(()), Err)
    }
}

impl FromStr for Porcelain {
    type Err = Kept;

    fn from_str(reply: &str) -> Result<Self, Self::Err> {
        reply
            .split(Self::SEPARATOR)
            .filter(|entry| !entry.is_empty())
            .map(str::parse)
            .collect::<Result<_, _>>()
            .map(Self)
    }
}
