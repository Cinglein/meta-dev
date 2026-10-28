mod step;

use std::path::Path;
use std::time::{Duration, Instant};

use meta_run::Unread;
use serde::Deserialize;

use crate::golden::Golden;
use crate::golden_failure::GoldenFailure;
use crate::places::Places;
use crate::recording::Recording;
use step::ScenarioStep;

#[derive(Debug, Deserialize)]
pub struct Scenario {
    step: Vec<ScenarioStep>,
}

impl Scenario {
    const PATIENCE: Duration = Duration::from_secs(60);

    pub fn recorded(&self, into: &Path, bin: &Path) -> Result<Vec<Recording>, GoldenFailure> {
        let unreadable = |source| GoldenFailure::Unreadable { path: into.to_path_buf(), source };
        let root = tempfile::tempdir().map_err(unreadable)?;
        let places = Places::new(root.path(), bin);
        let unbuilt = |unread| GoldenFailure::Unbuilt { scenario: into.to_path_buf(), unread };
        let recorded = self.step.iter().enumerate().map(|(index, step)| {
            let ran = step.run(&places).ran(Instant::now() + Self::PATIENCE);
            let ran = ran.map_err(|failure| unbuilt(Unread::from(failure)))?;
            match step.record() {
                Some(name) => Ok(Some(Recording::ran(into.join(format!("{index:02}-{name}")), &ran, &places))),
                None if ran.success() => Ok(None),
                None => Err(unbuilt(Unread::Failed(ran.status(), ran.stderr().clone()))),
            }
        });
        recorded.filter_map(Result::transpose).collect()
    }
}

impl TryFrom<&Path> for Scenario {
    type Error = GoldenFailure;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let text = Golden::new(path).text()?;
        let malformed = |failure: toml::de::Error| GoldenFailure::Malformed {
            path: path.to_path_buf(),
            reason: failure.to_string(),
        };
        toml::from_str(&text).map_err(malformed)
    }
}
