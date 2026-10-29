mod touched_file;

use std::path::Path;

use serde::{Deserialize, Serialize};
use strum::Display;

use touched_file::TouchedFile;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum BuildStart {
    Clean,
    #[strum(to_string = "touched {0}")]
    Touched(TouchedFile),
}

impl BuildStart {
    #[must_use]
    pub fn touched(&self) -> Option<&Path> {
        match self {
            BuildStart::Clean => None,
            BuildStart::Touched(file) => Some(file.path()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::BuildStart;
    use crate::budgets::Measure;
    use crate::policy::Policy;

    const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

    #[test]
    fn the_policy_builds_clean_first_then_touches_files_that_exist() {
        let builds = Policy::with_added("").unwrap().budgets().builds();
        let starts: Vec<&BuildStart> = builds.iter().filter_map(Measure::build_start).collect();
        assert_eq!(starts.first(), Some(&&BuildStart::Clean));
        let existing = starts
            .iter()
            .filter_map(|start| start.touched())
            .filter(|file| Path::new(ROOT).join(file).is_file());
        assert_eq!(existing.count() + 1, starts.len());
    }
}
