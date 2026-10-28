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
    const HEAD: &str = "diff --git a/a.txt b/a.txt\nindex 1..2 100644\n--- a/a.txt\n+++ b/a.txt\n";
    const NEXT: &str = "diff --git a/b.txt b/b.txt\nnew file mode 100644\n--- /dev/null\n+++ b/b.txt\n";
    const REMOVED: &str = concat!("-/Users/", "alice/x\n");
    const FIRST_HUNK: &str = "@@ -1,0 +2,1 @@\n";
    const LATER_HUNK: &str = "@@ -9 +10,3 @@\n";
    const WHERE: &str = "b.txt:12: scratch path in a staged line";

    fn refusal(diff: &str) -> Option<String> {
        diff.parse::<StagedDiff>().ok()?.refusal().map(|why| why.to_string())
    }

    #[test]
    fn the_first_scratch_or_home_path_among_added_lines_is_named_by_file_and_line() {
        let diff = format!("{HEAD}{FIRST_HUNK}{CLEAN}{LATER_HUNK}{CLEAN}{CLEAN}{NEXT}{LATER_HUNK}{CLEAN}{CLEAN}{LEAK}");
        let first = refusal(&diff).unwrap_or_default();
        assert!(first.starts_with(WHERE), "{first}");
    }

    #[test]
    fn removed_lines_clean_additions_and_an_empty_diff_are_not_refused() {
        let removed = format!("{HEAD}{FIRST_HUNK}{REMOVED}{CLEAN}{NEXT}{LATER_HUNK}{REMOVED}");
        assert!([removed, String::new()].iter().all(|diff| refusal(diff).is_none()));
    }
}
