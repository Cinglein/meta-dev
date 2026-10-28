mod scenario;

use std::path::{Path, PathBuf};

use crate::golden_failure::GoldenFailure;
use crate::recording::Recording;
use scenario::Scenario;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Goldens(PathBuf);

impl Goldens {
    const DIR: &'static str = "../../goldens";
    const SCENARIO: &'static str = "toml";

    #[must_use]
    pub fn workspace() -> Self {
        Self(Path::new(env!("CARGO_MANIFEST_DIR")).join(Self::DIR))
    }

    #[must_use]
    pub fn new(dir: &Path) -> Self {
        Self(dir.to_path_buf())
    }

    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.0
    }

    pub fn recordings(&self) -> Result<Vec<Recording>, GoldenFailure> {
        let recorded = self.scenarios()?.into_iter().map(|scenario| Recording::all_in(&scenario.with_extension("")));
        Ok(recorded.collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect())
    }

    pub fn refreshed(&self, bin: &Path) -> Result<(), GoldenFailure> {
        self.scenarios()?.iter().try_for_each(|path| {
            let dir = path.with_extension("");
            let recordings = Scenario::try_from(path.as_path())?.recorded(&dir, bin)?;
            Recording::cleared(&dir);
            recordings.iter().try_for_each(Recording::written)
        })
    }

    pub(crate) fn listed(dir: &Path, extension: &str) -> Result<Vec<PathBuf>, GoldenFailure> {
        let unreadable = |source| GoldenFailure::Unreadable { path: dir.to_path_buf(), source };
        let entries = std::fs::read_dir(dir).map_err(unreadable)?.collect::<Result<Vec<_>, _>>().map_err(unreadable)?;
        let mut listed: Vec<PathBuf> = entries.into_iter().map(|entry| entry.path()).collect();
        listed.retain(|path| path.extension().is_some_and(|found| found == extension));
        listed.sort();
        Ok(listed)
    }

    fn scenarios(&self) -> Result<Vec<PathBuf>, GoldenFailure> {
        Self::listed(&self.0, Self::SCENARIO)
    }
}
