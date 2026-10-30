use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transport {
    Local(PathBuf),
    Command,
}

impl Transport {
    const SCHEME: &'static str = "://";
    const HOST: char = ':';
    const DIRECTORY: char = '/';
    const SUFFIXES: [&'static str; 4] = ["/.git", "", ".git/.git", ".git"];

    #[must_use]
    pub fn below(self, directory: &Path) -> Transport {
        match self {
            Transport::Local(path) => Transport::Local(directory.join(path)),
            Transport::Command => Transport::Command,
        }
    }

    pub(crate) fn opened(&self) -> Option<git2::Repository> {
        match self {
            Transport::Local(path) => Self::SUFFIXES.iter().find_map(|suffix| Self::suffixed(path, suffix)),
            Transport::Command => None,
        }
    }

    fn suffixed(path: &Path, suffix: &str) -> Option<git2::Repository> {
        let mut candidate = path.as_os_str().to_owned();
        candidate.push(suffix);
        git2::Repository::open_ext(candidate, git2::RepositoryOpenFlags::NO_SEARCH, std::iter::empty::<&Path>()).ok()
    }
}

impl FromStr for Transport {
    type Err = Infallible;

    fn from_str(url: &str) -> Result<Transport, Infallible> {
        let hosted = url.split_once(Self::HOST).filter(|(before, _)| !before.contains(Self::DIRECTORY));
        Ok(match hosted {
            Some(_) => Transport::Command,
            None if url.is_empty() || url.contains(Self::SCHEME) => Transport::Command,
            None => Transport::Local(PathBuf::from(url)),
        })
    }
}
