mod patch;

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use meta_log::RemoteName;

use crate::request::output::Listed;
use patch::Patch;

#[derive(Debug)]
pub struct PushedCommits {
    directory: PathBuf,
    remote: RemoteName,
}

impl PushedCommits {
    const QUOTE_PATH: [(&'static str, &'static str); 3] =
        [("GIT_CONFIG_COUNT", "1"), ("GIT_CONFIG_KEY_0", "core.quotePath"), ("GIT_CONFIG_VALUE_0", "false")];
    const UNPUSHED: [&'static str; 3] = ["log", "HEAD", "--not"];
    const PATCH: [&'static str; 9] = [
        "--cc",
        "-M",
        "--text",
        "-U0",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--src-prefix=a/",
        "--dst-prefix=b/",
    ];
    const COMMIT_LINE: &'static str = "--format==commit %h";

    pub fn refusal_in(directory: &Path, remote: &RemoteName) -> Option<Listed> {
        let pushed = Self { directory: directory.to_owned(), remote: remote.clone() };
        pushed.scan().err()
    }

    fn scan(&self) -> Result<(), Listed> {
        let output = self.log()?;
        let Ok(patch) = String::from_utf8_lossy(&output.stdout).parse::<Patch>();
        patch.first_leak().map_or(Ok(()), |leak| Err(Listed::new(leak)))
    }

    fn log(&self) -> Result<Output, Listed> {
        let shown = Command::new("git")
            .arg("-C")
            .arg(&self.directory)
            .args(Self::UNPUSHED)
            .arg(format!("--remotes={}", self.remote))
            .args(Self::PATCH)
            .arg(Self::COMMIT_LINE)
            .envs(Self::QUOTE_PATH)
            .output();
        let unreadable = |why: String| Listed::new(format!("cannot read {self}: {why}; not pushed"));
        let output = shown.map_err(|error| unreadable(error.to_string()))?;
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        output.status.success().then_some(output).ok_or_else(|| unreadable(stderr))
    }
}

impl fmt::Display for PushedCommits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "the commits to push from {}", self.directory.display())
    }
}

#[cfg(test)]
mod tests {
    use super::PushedCommits;

    #[test]
    fn a_directory_git_cannot_read_holds_the_push_and_says_why() {
        let empty = tempfile::tempdir().unwrap();
        let held = PushedCommits::refusal_in(empty.path(), &"origin".parse().unwrap()).map(|why| why.to_string());
        assert!(
            held.is_some_and(|why| why.starts_with("cannot read the commits to push") && why.ends_with("not pushed"))
        );
    }
}
