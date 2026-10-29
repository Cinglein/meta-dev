mod patch;

use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Instant;

use meta_log::RemoteName;
use meta_run::Run;

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

    pub fn refusal_in(directory: &Path, remote: &RemoteName, deadline: Instant) -> Option<Listed> {
        let pushed = Self { directory: directory.to_owned(), remote: remote.clone() };
        pushed.scan(deadline).err()
    }

    fn scan(&self, deadline: Instant) -> Result<(), Listed> {
        let shown = self.log(deadline)?;
        let Ok(patch) = shown.parse::<Patch>();
        patch.first_leak().map_or(Ok(()), |leak| Err(Listed::new(leak)))
    }

    fn log(&self, deadline: Instant) -> Result<String, Listed> {
        let run = Run::git(&self.directory).args(Self::UNPUSHED).arg(format!("--remotes={}", self.remote));
        let run = run.args(Self::PATCH).arg(Self::COMMIT_LINE);
        let run = Self::QUOTE_PATH.into_iter().fold(run, |run, (name, value)| run.var(name, value));
        run.stdout(deadline).map_err(|failure| Listed::new(format!("cannot read {self}: {failure}; not pushed")))
    }
}

impl fmt::Display for PushedCommits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "the commits to push from {}", self.directory.display())
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::PushedCommits;

    const PATIENT: Duration = Duration::from_secs(10);

    #[test]
    fn a_directory_git_cannot_read_holds_the_push_and_says_why() {
        let empty = tempfile::tempdir().unwrap();
        let held = PushedCommits::refusal_in(empty.path(), &"origin".parse().unwrap(), Instant::now() + PATIENT);
        let held = held.map(|why| why.to_string());
        assert!(
            held.is_some_and(|why| why.starts_with("cannot read the commits to push") && why.ends_with("not pushed"))
        );
    }
}
