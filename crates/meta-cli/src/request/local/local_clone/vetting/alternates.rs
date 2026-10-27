use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alternates {
    objects: PathBuf,
    listed: Vec<PathBuf>,
}

impl Alternates {
    const COMMENT: char = '#';

    pub fn new(objects: PathBuf, listed: &str) -> Self {
        let Ok(parsed) = listed.parse::<Self>();
        Self { objects, ..parsed }
    }

    pub fn lends_into(&self, path: &Path) -> Option<()> {
        self.listed
            .iter()
            .map(|line| Self::resolved(&self.objects.join(line)))
            .any(|lent| lent.starts_with(path))
            .then_some(())
    }

    pub fn resolved(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), |resolved, component| match component {
            Component::ParentDir => resolved.parent().map(Path::to_path_buf).unwrap_or_default(),
            Component::CurDir => resolved,
            other => resolved.join(other),
        })
    }
}

impl FromStr for Alternates {
    type Err = std::convert::Infallible;

    fn from_str(listed: &str) -> Result<Self, Self::Err> {
        let lines = listed.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with(Self::COMMENT));
        Ok(Self { objects: PathBuf::new(), listed: lines.map(PathBuf::from).collect() })
    }
}
