use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Transport {
    Local(PathBuf),
    Command,
}

impl Transport {
    const SCHEME: &'static str = "://";
    const HOST: char = ':';
    const DIRECTORY: char = '/';

    pub(crate) fn local_path(&self, directory: &Path) -> Option<PathBuf> {
        match self {
            Transport::Local(path) => Some(directory.join(path)),
            Transport::Command => None,
        }
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
