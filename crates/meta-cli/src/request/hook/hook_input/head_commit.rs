use std::ffi::OsStr;
use std::fmt;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::request::output::Listed;

#[derive(Debug)]
pub struct HeadCommit(PathBuf);

impl HeadCommit {
    const SCRATCH: &'static str = concat!("/tmp/", "claude-");
    const LITERAL_PATHSPECS: (&'static str, &'static str) = ("GIT_LITERAL_PATHSPECS", "1");
    const FILES: [&'static str; 7] = ["diff-tree", "-r", "-z", "--root", "--no-commit-id", "--name-only", "HEAD"];
    const ADDED: [&'static str; 7] =
        ["show", "--format=", "-U0", "--no-color", "--no-ext-diff", "--no-renames", "HEAD"];
    const HUNK: &'static str = "@@ ";

    pub fn refusal_in(directory: &Path) -> Option<Listed> {
        let head = Self(directory.to_owned());
        head.scan().err()
    }

    fn scan(&self) -> Result<(), Listed> {
        let files = self.git(&Self::FILES.map(OsStr::new))?;
        files.stdout.split(|byte| *byte == b'\0').filter(|name| !name.is_empty()).try_for_each(|name| {
            let file = OsStr::from_bytes(name);
            let words = [Self::ADDED.map(OsStr::new).as_slice(), &[OsStr::new("--"), file]].concat();
            let shown = self.git(&words)?;
            Self::leak(file, &String::from_utf8_lossy(&shown.stdout)).map_or(Ok(()), Err)
        })
    }

    fn git(&self, words: &[&OsStr]) -> Result<Output, Listed> {
        let (variable, literal) = Self::LITERAL_PATHSPECS;
        let shown = Command::new("git").arg("-C").arg(&self.0).args(words).env(variable, literal).output();
        let unreadable = |why: String| Listed::new(format!("cannot read {self}: {why}; not pushed"));
        let output = shown.map_err(|error| unreadable(error.to_string()))?;
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        output.status.success().then_some(output).ok_or_else(|| unreadable(stderr))
    }

    fn added_range(line: &str) -> Option<&str> {
        line.strip_prefix(Self::HUNK)?.split_whitespace().find_map(|word| word.strip_prefix('+'))
    }

    fn leak(file: &OsStr, diff: &str) -> Option<Listed> {
        let line = diff
            .lines()
            .skip_while(|line| !line.starts_with(Self::HUNK))
            .scan(0, |next, line| {
                *next = Self::added_range(line)
                    .and_then(|range| range.split(',').next())
                    .and_then(|first| first.parse().ok())
                    .unwrap_or(*next);
                let added = line.strip_prefix('+');
                let at = *next;
                *next += usize::from(added.is_some());
                Some(added.filter(|text| text.contains(Self::SCRATCH)).map(|_| at))
            })
            .flatten()
            .next()?;
        Some(Listed::new(format!(
            "{}:{line}: agent scratch path in the commit; remove it, git commit --amend; not pushed",
            file.to_string_lossy()
        )))
    }
}

impl fmt::Display for HeadCommit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "HEAD in {}", self.0.display())
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::HeadCommit;

    const DIFF: &str = concat!(
        "diff --git a/f b/f\n--- a/f\n+++ b/f\n@@ -1,0 +2,2 @@\n+/home/runner/x\n+cd /tmp/",
        "claude-501/x\n@@ -9 +11 @@\n-old\n+/tmp/",
        "claude-1\n"
    );
    const REMOVED: &str = concat!("+++ b/tmp/", "claude-1\n@@ -3 +2,0 @@\n-/tmp/", "claude-1\n");
    const NAMED: &str = "f:3: agent scratch path";

    #[test]
    fn the_first_added_scratch_line_is_numbered_by_the_new_file_and_removals_and_home_paths_pass() {
        let leak = |diff: &str| HeadCommit::leak(OsStr::new("f"), diff).map(|found| found.to_string());
        assert!(leak(DIFF).is_some_and(|found| found.starts_with(NAMED)));
        assert!(leak(REMOVED).is_none());
    }

    #[test]
    fn a_directory_git_cannot_read_holds_the_push_and_says_why() {
        let empty = tempfile::tempdir().unwrap();
        let held = HeadCommit::refusal_in(empty.path()).map(|why| why.to_string());
        assert!(held.is_some_and(|why| why.starts_with("cannot read HEAD in") && why.ends_with("not pushed")));
    }
}
