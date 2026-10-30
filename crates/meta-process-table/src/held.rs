use std::path::PathBuf;

use super::scan::Scan;
use super::uninspected::Uninspected;

#[derive(Debug, Default)]
pub struct Held {
    paths: Vec<PathBuf>,
    uninspected: Uninspected,
}

impl Held {
    #[must_use]
    pub fn uninspected(&self) -> Uninspected {
        self.uninspected
    }

    #[must_use]
    pub fn under(self, scanned: &[PathBuf]) -> Vec<PathBuf> {
        self.paths.into_iter().filter(|path| scanned.iter().any(|root| path.starts_with(root))).collect()
    }

    pub(crate) fn holding(self, paths: Vec<PathBuf>) -> Self {
        Self { paths: [self.paths, paths].concat(), ..self }
    }

    pub(crate) fn refused(self) -> Self {
        Self { uninspected: self.uninspected.counted(), ..self }
    }
}

impl FromIterator<Scan> for Held {
    fn from_iter<Scans: IntoIterator<Item = Scan>>(scans: Scans) -> Self {
        scans.into_iter().fold(Self::default(), |held, scan| scan.added(held))
    }
}
