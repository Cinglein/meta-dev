use std::convert::Infallible;
use std::path::PathBuf;
use std::str::FromStr;

use task::SpecSettings;

#[derive(Debug)]
pub struct CheckedFile(Vec<PathBuf>);

impl CheckedFile {
    pub fn named(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn among(self, changed: &[PathBuf]) -> Vec<PathBuf> {
        self.0.into_iter().filter(|file| changed.contains(file)).collect()
    }
}

impl FromStr for CheckedFile {
    type Err = Infallible;

    fn from_str(line: &str) -> Result<CheckedFile, Infallible> {
        let checked = line.split_once(SpecSettings::CHECKED).map(|(_, checked)| checked);
        let header =
            checked.map(|checked| checked.split_once(SpecSettings::WORDING).map_or(checked, |(header, _)| header));
        let files = header.into_iter().flat_map(|header| header.split(SpecSettings::ALSO_CHECKED));
        Ok(CheckedFile(files.filter_map(|enforcer| enforcer.split_whitespace().next()).map(PathBuf::from).collect()))
    }
}
