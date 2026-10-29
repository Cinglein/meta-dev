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
    const SHOWN: &str = "clean, touched crates/meta-cli/src/lib.rs, touched crates/meta-cli/tests/status.rs, touched crates/meta-schema/src/lib.rs";

    #[test]
    fn the_policy_builds_clean_first_then_touches_files_that_exist() {
        let builds = Policy::with_added("").unwrap().budgets().builds();
        let starts: Vec<&BuildStart> = builds.iter().filter_map(Measure::build_start).collect();
        let shown: Vec<String> = starts.iter().map(ToString::to_string).collect();
        assert_eq!(shown.join(", "), SHOWN);
        assert!(
            starts
                .iter()
                .filter_map(|start| start.touched())
                .all(|file| Path::new(ROOT).join(file).is_file())
        );
    }
}
