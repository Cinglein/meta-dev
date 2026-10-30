mod redirect;

use std::path::{Path, PathBuf};

use redirect::Redirect;

use super::tokens::Segment;
use super::working_directory::WorkingDirectory;

pub type GhRepositories = fn(&[String]) -> Option<Vec<Option<String>>>;

#[derive(Debug, PartialEq, Eq)]
pub enum Pin {
    Directory(PathBuf),
    Repository(String),
    Unpinned,
}

impl Pin {
    pub fn of_calls(parts: &[&Segment], cwd: &Path, gh_repositories: GhRepositories) -> Vec<Pin> {
        let first_lost = parts
            .iter()
            .position(|part| Redirect::unfollowable(part).or_else(|| Redirect::inherited(part.words())).is_some());
        let any_command = |_: &[String]| Some(());
        let mut shell = WorkingDirectory::new(cwd);
        parts
            .iter()
            .enumerate()
            .filter_map(|(index, part)| {
                let directory = shell.follow(part.words(), any_command)?;
                let lost = first_lost.filter(|first| index >= *first).map(|_| ()).or_else(|| Redirect::local(part));
                Some(Self::pins(gh_repositories(part.words()), directory, lost))
            })
            .flatten()
            .collect()
    }

    pub fn directory(&self) -> Option<&PathBuf> {
        match self {
            Self::Directory(directory) => Some(directory),
            Self::Repository(_) | Self::Unpinned => None,
        }
    }

    pub fn repository(&self) -> Option<&String> {
        match self {
            Self::Repository(name) => Some(name),
            Self::Directory(_) | Self::Unpinned => None,
        }
    }

    fn pins(named: Option<Vec<Option<String>>>, directory: PathBuf, lost: Option<()>) -> Vec<Pin> {
        let named = named.filter(|named| !named.is_empty());
        match (named, lost) {
            (Some(named), _) => named.into_iter().map(|name| name.map_or(Pin::Unpinned, Pin::Repository)).collect(),
            (None, Some(())) => vec![Pin::Unpinned],
            (None, None) => vec![Pin::Directory(directory)],
        }
    }
}
