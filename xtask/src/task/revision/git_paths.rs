use std::convert::Infallible;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitPaths(Vec<PathBuf>);

impl GitPaths {
    const SEPARATOR: char = '\0';

    pub fn and(mut self, more: GitPaths) -> GitPaths {
        self.0.extend(more.0);
        self
    }
}

impl FromIterator<PathBuf> for GitPaths {
    fn from_iter<Paths: IntoIterator<Item = PathBuf>>(paths: Paths) -> GitPaths {
        GitPaths(paths.into_iter().collect())
    }
}

impl IntoIterator for GitPaths {
    type Item = PathBuf;
    type IntoIter = std::vec::IntoIter<PathBuf>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromStr for GitPaths {
    type Err = Infallible;

    fn from_str(listed: &str) -> Result<GitPaths, Infallible> {
        Ok(GitPaths(listed.split(Self::SEPARATOR).filter(|path| !path.is_empty()).map(PathBuf::from).collect()))
    }
}
