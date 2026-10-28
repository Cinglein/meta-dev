mod debugged;
mod mode;

use std::path::{Path, PathBuf};

use crate::difference::Difference;
use crate::golden_failure::GoldenFailure;
use crate::goldens::Goldens;
use mode::Mode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Golden(PathBuf);

impl Golden {
    pub const BLESS: &'static str = Mode::BLESS;
    const SNAPSHOTS: &'static str = "snapshots";
    const MODULE_SEPARATOR: &'static str = "::";
    const SNAPSHOT: &'static str = "snap";

    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self(path.to_path_buf())
    }

    #[must_use]
    pub fn for_test(module: &str, name: &str) -> Self {
        let root = Goldens::workspace().dir().join(Self::SNAPSHOTS);
        let dir = module.split(Self::MODULE_SEPARATOR).fold(root, |dir, part| dir.join(part));
        Self(dir.join(name).with_extension(Self::SNAPSHOT))
    }

    pub fn checked(&self, actual: &str) -> Result<(), GoldenFailure> {
        match Mode::current()? {
            Mode::Bless => self.clone().written(actual),
            Mode::Check => {
                let golden = self.text()?;
                let differs = |difference| GoldenFailure::Differs { path: self.0.clone(), difference };
                Difference::between(&golden, actual).map_or(Ok(()), |difference| Err(differs(difference)))
            }
        }
    }

    pub fn written(self, text: &str) -> Result<(), GoldenFailure> {
        let dir = self.0.parent().unwrap_or(Path::new(""));
        let written = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&self.0, text));
        written.map_err(|source| GoldenFailure::Unreadable { path: self.0, source })
    }

    pub fn text(&self) -> Result<String, GoldenFailure> {
        std::fs::read_to_string(&self.0).map_err(|source| GoldenFailure::Unreadable { path: self.0.clone(), source })
    }
}
