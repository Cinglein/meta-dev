use std::path::{Path, PathBuf};

use super::git_call::GitCall;
use super::working_directory::WorkingDirectory;

pub type GhRepositories = fn(&[String]) -> Option<Vec<Option<String>>>;

#[derive(Debug, PartialEq, Eq)]
pub enum Pin {
    Directory(PathBuf),
    Repository(String),
    Unpinned,
}

impl Pin {
    const DIRECTORY_CHANGERS: &'static [&'static str] = &["cd", "pushd", "popd"];
    const FOLLOWED: &'static str = "cd";
    const HOMES: &'static [&'static str] = &["$HOME", "$PWD"];
    const UNFOLLOWABLE: &'static [char] = &['$', '*', '?', '[', '`', '{', '\\', '!'];

    pub fn of_calls(segments: &[&[String]], cwd: &Path, gh_repositories: GhRepositories) -> Vec<Pin> {
        let lost = segments.iter().position(|words| Self::unfollowable(words).is_some());
        let calls = |words: &[String]| GitCall::parse(words).map(|_| ()).or(gh_repositories(words).map(|_| ()));
        let mut shell = WorkingDirectory::new(cwd);
        segments
            .iter()
            .enumerate()
            .filter_map(|(index, words)| {
                let directory = shell.follow(words, calls)?;
                let lost = lost.filter(|lost| index >= *lost).map(|_| ());
                Some(Self::pins(gh_repositories(words), directory, lost))
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

    fn unfollowable(words: &[String]) -> Option<&String> {
        let changer = words.iter().find(|word| Self::DIRECTORY_CHANGERS.contains(&word.as_str()))?;
        Self::plain_target(words).is_none().then_some(changer)
    }

    fn plain_target(words: &[String]) -> Option<&str> {
        match words {
            [cd, target] if cd == Self::FOLLOWED => {
                let bare = Self::HOMES.iter().find_map(|home| target.strip_prefix(home)).unwrap_or(target);
                Some(target.as_str()).filter(|target| !target.starts_with('-') && !bare.contains(Self::UNFOLLOWABLE))
            }
            _ => None,
        }
    }
}
