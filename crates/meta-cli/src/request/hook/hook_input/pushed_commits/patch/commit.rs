mod file_diff;

use std::convert::Infallible;
use std::str::FromStr;

use super::leak::Leak;
use file_diff::FileDiff;

#[derive(Debug)]
pub struct Commit {
    abbreviation: String,
    files: String,
}

impl Commit {
    const FILE_START: &'static str = "\ndiff ";

    pub fn first_leak(&self) -> Option<Leak> {
        self.files.split(Self::FILE_START).skip(1).find_map(|text| {
            let Ok(file) = text.parse::<FileDiff>();
            file.first_leak(&self.abbreviation)
        })
    }
}

impl FromStr for Commit {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (abbreviation, files) = text.split_once('\n').unwrap_or((text, ""));
        Ok(Self { abbreviation: abbreviation.to_owned(), files: format!("\n{files}") })
    }
}
