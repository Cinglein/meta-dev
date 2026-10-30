use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::source_path::SourcePath;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SmellLocation(String);

impl SmellLocation {
    const LINE: char = ':';

    #[must_use]
    pub fn file(&self) -> SourcePath {
        SourcePath::new(Path::new(self.0.split(Self::LINE).next().unwrap_or_default()))
    }
}
