mod git_call;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::request::output::Listed;
use crate::request::project::Config;
use git_call::GitCall;

#[derive(Debug)]
pub struct ShellCommand(Vec<Vec<String>>);

impl ShellCommand {
    const SEPARATORS: &'static [char] = &[';', '&', '|', '\n', '(', ')'];
    const QUOTES: &'static [char] = &['\'', '"'];
    const PR_MERGE: &'static [&'static str] = &["gh", "pr", "merge"];
    pub fn refusal(&self, config: &Config) -> Option<Listed> {
        self.segments()
            .find(|words| words.windows(Self::PR_MERGE.len()).any(|window| window == Self::PR_MERGE))
            .map(|_| Listed::new("gh pr merge is the owner's job"))
            .or_else(|| self.segments().filter_map(GitCall::parse).find_map(|call| call.push_refusal(config)))
    }

    pub fn commit_directory(&self, cwd: &Path) -> Option<PathBuf> {
        self.segments().scan(cwd.to_path_buf(), |directory, words| Some(Self::step(directory, words))).flatten().next()
    }

    pub fn commit_directories(&self, cwd: &Path) -> Vec<PathBuf> {
        self.commit_directory(cwd).into_iter().collect()
    }

    fn step(directory: &mut PathBuf, words: &[String]) -> Option<PathBuf> {
        match words {
            [cd, target] if cd == "cd" => {
                directory.push(target);
                None
            }
            _ => GitCall::parse(words)
                .filter(|call| call.writes_commits().is_some())
                .map(|call| call.directory(directory)),
        }
    }

    pub fn runs(&self, subcommand: &str) -> Option<()> {
        self.segments().filter_map(GitCall::parse).find_map(|call| call.runs(subcommand))
    }

    fn segments(&self) -> impl Iterator<Item = &[String]> {
        self.0.iter().map(Vec::as_slice)
    }
}

impl FromStr for ShellCommand {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            text.split(Self::SEPARATORS)
                .map(|segment| {
                    segment
                        .split_whitespace()
                        .map(|word| word.trim_matches(Self::QUOTES).to_owned())
                        .filter(|word| !word.is_empty())
                        .collect()
                })
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::ShellCommand;
    use crate::request::project::Config;

    const WRITERS: [&str; 4] = ["git revert HEAD", "git pull", "git am x.patch", "git commit-tree t"];
    const NOT_WRITERS: [&str; 3] = ["git merge --abort", "git rebase --abort", "git merge-base a b"];
    const CWD: &str = "/w/good";
    const CHAINS: [(&str, &[&str]); 5] = [
        ("(cd a && git -c x=y -C b commit -m 'cd z')", &["/w/good/a/b"]),
        ("git commit -m x && git -C ../bad commit -m y", &["/w/good", "/w/good/../bad"]),
        ("git commit; cd ../bad; git commit", &["/w/good", "/w/good/../bad"]),
        ("(cd ../bad && git commit) && git commit", &["/w/good/../bad", "/w/good"]),
        ("cd $PWD/a && git -C ~/b commit && git -C $HOME commit", &["~/b", "~"]),
    ];

    #[test]
    fn every_commit_writer_resolves_its_own_directory_and_aborts_write_nothing() {
        let home = std::env::var("HOME").unwrap();
        let directories = |command: &str| command.parse::<ShellCommand>().unwrap().commit_directories(Path::new(CWD));
        let expected =
            |paths: &[&str]| paths.iter().map(|path| PathBuf::from(path.replacen('~', &home, 1))).collect::<Vec<_>>();
        assert!(WRITERS.iter().all(|command| directories(command).len() == 1));
        assert!(NOT_WRITERS.iter().all(|command| directories(command).is_empty()));
        assert!(CHAINS.iter().all(|(command, paths)| directories(command) == expected(paths)));
    }

    const CONFIG: &str = "protected = [\"main\"]\n";
    const MERGES: [&str; 2] = ["gh pr merge 1", "cd x && gh pr merge 1 --squash"];
    const PUSHES: [&str; 6] = [
        "git push --force origin feature",
        "git push -uf origin feature",
        "git push origin +feature",
        "git -C x push origin HEAD:refs/heads/main",
        "git push --force-with-lease origin main",
        "git --no-pager push origin main",
    ];
    const ALLOWED: [&str; 4] = [
        "git push -u origin feature",
        "git push --force-with-lease origin feature",
        "git commit -m 'git push origin main'",
        "gh pr view 1",
    ];
    const PUSH_REASON: &str = "git push";
    const LOG_CONFIG: &str = "protected = [\"main\", \"meta/log\", \"meta/logs\"]\nlog_branches = [\"meta/log\"]\n";
    const LOG_PUSH: &str = "git push origin meta/log";
    const ONLY_META: &str = "only `meta` writes it";
    const FORCED: &str = "git push --force origin feature";
    const LEASE: &str = "lease";
    const NEAR_LOG: &str = "git push origin meta/logs";
    const OPEN_PR: &str = "open a PR";

    #[test]
    fn merges_force_pushes_and_protected_pushes_are_refused_and_nothing_else() {
        let config: Config = toml::from_str(CONFIG).unwrap();
        let refusal =
            |command: &str| command.parse::<ShellCommand>().unwrap().refusal(&config).map(|why| why.to_string());
        assert!(MERGES.iter().all(|command| refusal(command).is_some()));
        assert!(PUSHES.iter().all(|command| refusal(command).is_some_and(|why| why.starts_with(PUSH_REASON))));
        assert!(ALLOWED.iter().all(|command| refusal(command).is_none()));
    }

    #[test]
    fn a_log_push_names_meta_and_a_force_push_names_no_lease() {
        let config: Config = toml::from_str(LOG_CONFIG).unwrap();
        let refusal = |command: &str| command.parse::<ShellCommand>().unwrap().refusal(&config).unwrap().to_string();
        assert!(refusal(LOG_PUSH).ends_with(ONLY_META) && refusal(NEAR_LOG).ends_with(OPEN_PR));
        assert!(!refusal(FORCED).contains(LEASE));
    }
}
