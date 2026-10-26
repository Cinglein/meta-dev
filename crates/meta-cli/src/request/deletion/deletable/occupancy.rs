use std::path::Path;
use std::process::Command;

use crate::request::cli_failure::Message;
use crate::request::deletion::kept::Kept;

#[derive(Debug)]
pub struct Occupancy;

impl Occupancy {
    const NAMED: char = 'n';

    pub fn vacant(path: &Path) -> Result<(), Kept> {
        let listed = Command::new("lsof")
            .args(["-w", "-F", "n", "+D"])
            .arg(path)
            .output()
            .map_err(|failure| Kept::Uninspected(Message::new(failure)))?;
        match (
            String::from_utf8_lossy(&listed.stdout).lines().any(|line| line.starts_with(Self::NAMED)),
            listed.stderr.is_empty(),
        ) {
            (true, _) => Err(Kept::InUse),
            (false, true) => Ok(()),
            (false, false) => Err(Kept::Uninspected(Message::new(String::from_utf8_lossy(&listed.stderr)))),
        }
    }
}
