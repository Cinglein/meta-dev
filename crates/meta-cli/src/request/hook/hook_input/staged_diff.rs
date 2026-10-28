mod local_path;

use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use crate::request::output::Listed;
use local_path::LocalPath;

#[derive(Debug, PartialEq, Eq)]
pub struct StagedDiff(String);

impl StagedDiff {
    const STAGED: [&'static str; 7] =
        ["diff", "--cached", "--unified=0", "--no-color", "--no-ext-diff", "--src-prefix=a/", "--dst-prefix=b/"];
    const FILE_START: &'static str = "\n+++ b/";
    const HUNK_START: &'static str = "\n@@ ";

    pub fn refusal_in(directory: &Path) -> Option<Listed> {
        let shown = Command::new("git").arg("-C").arg(directory).args(Self::STAGED).output().ok()?;
        let Ok(diff) = String::from_utf8_lossy(&shown.stdout).parse::<StagedDiff>();
        diff.refusal()
    }

    fn refusal(&self) -> Option<Listed> {
        self.0.split(Self::FILE_START).skip(1).find_map(Self::in_file)
    }

    fn in_file(section: &str) -> Option<Listed> {
        let (file, hunks) = section.split_once(Self::HUNK_START)?;
        hunks.split(Self::HUNK_START).find_map(|hunk| Self::in_hunk(file, hunk))
    }

    fn in_hunk(file: &str, hunk: &str) -> Option<Listed> {
        let (header, body) = hunk.split_once('\n')?;
        let first: usize =
            header.split_whitespace().find_map(|word| word.strip_prefix('+'))?.split(',').next()?.parse().ok()?;
        body.lines().filter_map(|line| line.strip_prefix('+')).enumerate().find_map(|(offset, added)| {
            LocalPath::in_line(added).map(|found| {
                Listed::new(format!(
                    "{file}:{}: {found} in a staged line; use a relative path or drop it",
                    first + offset
                ))
            })
        })
    }
}

impl FromStr for StagedDiff {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(text.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::StagedDiff;

    const LEAK: &str = concat!("+cd /private/tmp/", "claude-501/x\n");
    const CLEAN: &str = "+clean\n";
    const REMOVED: &str = concat!("-/Users/", "alice/x\n");
    const A: &str = "diff --git a/a.txt b/a.txt\n--- a/a.txt\n+++ b/a.txt\n";
    const B: &str = "diff --git a/b.txt b/b.txt\n--- /dev/null\n+++ b/b.txt\n";
    const EARLY: &str = "@@ -1,0 +2,1 @@\n";
    const LATE: &str = "@@ -9 +10,3 @@\n";
    const WHERE: &str = "b.txt:12: scratch path in a staged line";

    #[test]
    fn the_first_scratch_or_home_path_among_added_lines_is_named_by_file_and_line_and_removals_pass() {
        let refusal = |diff: &str| diff.parse::<StagedDiff>().unwrap().refusal().map(|why| why.to_string());
        let leaked = format!("{A}{EARLY}{CLEAN}{LATE}{CLEAN}{CLEAN}{B}{LATE}{CLEAN}{CLEAN}{LEAK}");
        let removed = format!("{A}{EARLY}{REMOVED}{CLEAN}{B}{LATE}{REMOVED}");
        assert!(refusal(&leaked).is_some_and(|why| why.starts_with(WHERE)));
        assert!(refusal(&removed).is_none() && refusal("").is_none());
    }
}
