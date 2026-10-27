use std::convert::Infallible;
use std::path::PathBuf;
use std::str::FromStr;

use task::SpecSettings;

#[derive(Debug)]
pub struct CheckedFile(Option<PathBuf>);

impl CheckedFile {
    pub fn named(&self) -> bool {
        self.0.is_some()
    }

    pub fn among(self, changed: &[PathBuf]) -> Option<PathBuf> {
        self.0.filter(|file| changed.contains(file))
    }
}

impl FromStr for CheckedFile {
    type Err = Infallible;

    fn from_str(line: &str) -> Result<CheckedFile, Infallible> {
        let checked = line.split_once(SpecSettings::CHECKED).and_then(|(_, checked)| checked.split_once(' '));
        Ok(CheckedFile(checked.map(|(file, _)| PathBuf::from(file))))
    }
}
