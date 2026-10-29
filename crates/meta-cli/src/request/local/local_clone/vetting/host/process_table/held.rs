use std::path::PathBuf;

use super::scan::Scan;
use crate::request::local::local_clone::vetting::occupancy::Occupancy;
use crate::request::local::local_clone::vetting::uninspected::Uninspected;

#[derive(Debug, Default)]
pub struct Held {
    paths: Vec<PathBuf>,
    uninspected: Uninspected,
}

impl Held {
    pub fn uninspected(&self) -> Uninspected {
        self.uninspected
    }

    pub fn occupancy(self, scanned: &[PathBuf]) -> Occupancy {
        let under = self.paths.into_iter().filter(|path| scanned.iter().any(|root| path.starts_with(root)));
        Occupancy::Listed(under.collect())
    }

    pub fn holding(self, paths: Vec<PathBuf>) -> Self {
        Self { paths: [self.paths, paths].concat(), ..self }
    }

    pub fn refused(self) -> Self {
        Self { uninspected: self.uninspected.counted(), ..self }
    }
}

impl FromIterator<Scan> for Held {
    fn from_iter<Scans: IntoIterator<Item = Scan>>(scans: Scans) -> Self {
        scans.into_iter().fold(Self::default(), |held, scan| scan.added(held))
    }
}
