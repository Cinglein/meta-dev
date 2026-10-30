use std::fmt;
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
            origin @ ('+' | '-' | ' ') => [String::from(origin), content.into_owned()].concat(),
            'F' => Self::header(below, delta, &content),
            _ => content.into_owned(),
        }
    }

    fn header(below: &Path, delta: &git2::DiffDelta<'_>, text: &str) -> String {
        let relative = |file: git2::DiffFile<'_>| {
            let path = file.path().unwrap_or(Path::new(""));
            path.strip_prefix(below).unwrap_or(path).display().to_string()
        };
        let [old, new] = [relative(delta.old_file()), relative(delta.new_file())];
        text.lines().map(|line| Self::rebuilt(line, &old, &new) + Self::LINE_END).collect()
    }

    fn rebuilt(line: &str, old: &str, new: &str) -> String {
        let git = line.starts_with(Self::DIFF_GIT).then(|| format!("{}a/{old} b/{new}", Self::DIFF_GIT));
        let side = |prefixes: &[&str], path: &str| {
            prefixes.iter().find(|prefix| line.starts_with(**prefix)).map(|prefix| [prefix, path].concat())
        };
        git.or_else(|| side(&Self::OLD_SIDE, old))
            .or_else(|| side(&Self::NEW_SIDE, new))
            .unwrap_or_else(|| line.to_owned())
    }
}

impl fmt::Display for RelativeDiff {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "git diff --relative --merge-base {} in {}", self.base, self.root.display())
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
        let mut text = String::new();
        diff.print(git2::DiffFormat::Patch, |delta, _, line| {
            text.push_str(&Self::line(&below, &delta, &line));
            true
        })?;
        Ok(text)
    }
}
