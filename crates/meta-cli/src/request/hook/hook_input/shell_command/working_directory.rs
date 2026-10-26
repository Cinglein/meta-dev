use std::path::{Path, PathBuf};

use super::git_call::GitCall;

#[derive(Debug)]
pub struct WorkingDirectory {
    cwd: PathBuf,
    subshells: Vec<PathBuf>,
}

impl WorkingDirectory {
    const OPEN: &'static str = "(";
    const CLOSE: &'static str = ")";
    const CD: &'static str = "cd";
    const HOMES: &'static [&'static str] = &["~", "$HOME"];
    const PWD: &'static str = "$PWD";

    pub fn new(cwd: &Path) -> Self {
        Self { cwd: cwd.to_path_buf(), subshells: Vec::new() }
    }

    pub fn follow(&mut self, words: &[String], wanted: impl Fn(&[String]) -> Option<()>) -> Option<PathBuf> {
        match words {
            [open] if open == Self::OPEN => {
                self.subshells.push(self.current());
                None
            }
            [close] if close == Self::CLOSE => {
                self.subshells.pop();
                None
            }
            [cd, target] if cd == Self::CD => {
                let next = Self::resolve(&self.current(), Path::new(target));
                *self.subshells.last_mut().unwrap_or(&mut self.cwd) = next;
                None
            }
            _ => wanted(words).map(|()| Self::directory(&self.current(), words)),
        }
    }

    fn directory(current: &Path, words: &[String]) -> PathBuf {
        let directories = GitCall::parse(words).map(|call| call.directory_words().to_vec()).unwrap_or_default();
        directories.iter().fold(current.to_path_buf(), |directory, word| Self::resolve(&directory, Path::new(word)))
    }

    fn current(&self) -> PathBuf {
        self.subshells.last().unwrap_or(&self.cwd).clone()
    }

    fn resolve(base: &Path, target: &Path) -> PathBuf {
        let mut components = target.components();
        let head = components.next().map(|head| head.as_os_str().to_owned()).unwrap_or_default();
        let root = if Self::HOMES.iter().any(|home| head == *home) {
            std::env::var_os("HOME").map(PathBuf::from)
        } else {
            (head == Self::PWD).then(|| base.to_path_buf())
        };
        root.map_or_else(|| base.join(target), |root| root.join(components.as_path()))
    }
}
