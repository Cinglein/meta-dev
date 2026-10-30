use std::path::{Path, PathBuf};

use crate::git_failure::GitFailure;
use crate::query::Query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelativeDiff {
    base: String,
    root: PathBuf,
}

impl RelativeDiff {
    const HEAD: &'static str = "HEAD";
    const DIFF_GIT: &'static str = "diff --git ";
    const OLD_SIDE: [&'static str; 3] = ["--- a/", "rename from ", "copy from "];
    const NEW_SIDE: [&'static str; 3] = ["+++ b/", "rename to ", "copy to "];
    const LINE_END: &'static str = "\n";
    const SIDES: [&'static str; 2] = ["a/", " b/"];
    const CHAR_LEN: usize = 4;
    const OUTSIDE: &'static str = "the diff root is not inside the work tree";

    #[must_use]
    pub fn new(base: String, root: PathBuf) -> RelativeDiff {
        RelativeDiff { base, root }
    }

    fn below(&self, repository: &git2::Repository) -> Result<PathBuf, git2::Error> {
        let workdir = repository.workdir().and_then(|workdir| workdir.canonicalize().ok());
        let root = self.root.canonicalize().ok();
        let below =
            workdir.zip(root).and_then(|(workdir, root)| root.strip_prefix(workdir).ok().map(Path::to_path_buf));
        below.ok_or_else(|| git2::Error::from_str(Self::OUTSIDE))
    }

    fn line(below: &Path, delta: &git2::DiffDelta<'_>, line: &git2::DiffLine<'_>) -> String {
        let content = String::from_utf8_lossy(line.content());
        match line.origin() {
            origin @ ('+' | '-' | ' ') => {
                let mut sign = [0; Self::CHAR_LEN];
                [&*origin.encode_utf8(&mut sign), content.as_ref()].concat()
            }
            'F' => Self::header(below, delta, &content),
            _ => content.into_owned(),
        }
    }

    fn header(below: &Path, delta: &git2::DiffDelta<'_>, text: &str) -> String {
        let relative = |file: git2::DiffFile<'_>| {
            let path = file.path().unwrap_or(Path::new(""));
            path.strip_prefix(below).unwrap_or(path).to_string_lossy().into_owned()
        };
        let [old, new] = [relative(delta.old_file()), relative(delta.new_file())];
        text.lines().map(|line| Self::rebuilt(line, &old, &new) + Self::LINE_END).collect()
    }

    fn rebuilt(line: &str, old: &str, new: &str) -> String {
        let [old_side, new_side] = Self::SIDES;
        let git = line.starts_with(Self::DIFF_GIT).then(|| [Self::DIFF_GIT, old_side, old, new_side, new].concat());
        let side = |prefixes: &[&str], path: &str| {
            prefixes.iter().find(|prefix| line.starts_with(**prefix)).map(|prefix| [prefix, path].concat())
        };
        git.or_else(|| side(&Self::OLD_SIDE, old))
            .or_else(|| side(&Self::NEW_SIDE, new))
            .unwrap_or_else(|| line.to_owned())
    }
}

impl Query for RelativeDiff {
    type View = String;

    fn answered(&self, repository: &git2::Repository) -> Result<String, GitFailure> {
        let commit = |spec: &str| repository.revparse_single(spec).and_then(|object| object.peel_to_commit());
        let base = repository.merge_base(commit(Self::HEAD)?.id(), commit(&self.base)?.id())?;
        let below = self.below(repository)?;
        let mut options = git2::DiffOptions::new();
        Some(&below).filter(|below| !below.as_os_str().is_empty()).map(|below| options.pathspec(below));
        let tree = repository.find_commit(base)?.tree()?;
        let diff = repository.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut options))?;
        let mut lines = Vec::new();
        diff.print(git2::DiffFormat::Patch, |delta, _, line| {
            lines.push(Self::line(&below, &delta, &line));
            true
        })?;
        Ok(lines.concat())
    }
}
