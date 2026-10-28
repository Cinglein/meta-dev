mod commit;
mod leak;
mod line;

use std::convert::Infallible;
use std::str::FromStr;

use commit::Commit;
pub use leak::Leak;

#[derive(Debug)]
pub struct Patch(String);

impl Patch {
    const COMMIT_START: &'static str = "\n=commit ";

    pub fn first_leak(&self) -> Option<Leak> {
        self.0.split(Self::COMMIT_START).skip(1).find_map(|text| {
            let Ok(commit) = text.parse::<Commit>();
            commit.first_leak()
        })
    }
}

impl FromStr for Patch {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(format!("\n{text}")))
    }
}

#[cfg(test)]
mod tests {
    use super::Patch;

    const COMMITS: &str = concat!(
        "=commit c1\n\ndiff --git a/f b/f\n--- a/f\n+++ b/tmp/",
        "claude-d/f\n@@ -1,0 +4,2 @@\n+/home/runner/x\n+cd /tmp/",
        "claude-1\n=commit c2\n\ndiff --git a/g b/g\n+++ b/g\n@@ -9 +11 @@\n-/tmp/",
        "claude-0\n"
    );
    const MERGE: &str = concat!(
        "=commit m1\n\ndiff --cc g\n+++ b/g\n@@@ -1 -1 +1,3 @@@\n + /tmp/",
        "claude-2\n- /tmp/",
        "claude-3\n  ok\n++/tmp/",
        "claude-4\n"
    );
    const WHERE: &str = "tmp/claude-d/f:5: agent scratch path in commit c1";
    const MERGED: &str = "g:3: agent scratch path in commit m1";

    #[test]
    fn only_lines_added_by_every_parent_name_their_new_file_line_and_removals_and_headers_pass() {
        let leak = |text: &str| text.parse::<Patch>().unwrap().first_leak().map(|found| found.to_string());
        assert!(leak(COMMITS).is_some_and(|found| found.starts_with(WHERE)));
        assert!(leak(MERGE).is_some_and(|found| found.starts_with(MERGED)));
        assert!(leak("").is_none());
    }
}
