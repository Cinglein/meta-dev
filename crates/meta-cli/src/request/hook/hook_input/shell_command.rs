#[cfg(test)]
mod breaches;
mod git_call;
mod tokens;
mod working_directory;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use meta_schema::Actor;

use crate::request::output::Listed;
use crate::request::project::Config;
use crate::request::scale_policy::ShellRules;
use git_call::GitCall;
use tokens::{Kind, Segment, Tokens};
use working_directory::WorkingDirectory;

#[derive(Debug)]
pub struct ShellCommand(Tokens);

impl ShellCommand {
    const SHELL_RULE: &'static str = "shell rule:";
    const USE: &'static str =
        "is not allowed; use the Write or Edit tool, `git restore`, or `git rm` inside a code change";
    const GIT_USE: &'static str = "is not allowed for agents; commit on your branch, push normally, `git restore`";
    const EXPANDED: &'static str =
        "the program or the command it runs is built by $ expansion or substitution; write it out so it can be checked";
    const STDIN: &'static str = "reads its commands from stdin; pass them with -c so they can be checked";
    const PR_MERGE: &'static [&'static str] = &["gh", "pr", "merge"];
    const PR_CREATE: &'static [&'static str] = &["gh", "pr", "create"];
    const META: &'static str = "meta";
    const QUEUE_RUN: [&'static str; 2] = ["queue", "run"];
    const LANDING: &'static str =
        "meta queue run lands PRs: set META_ACTOR to an actor in tooling/policy.toml [queue] landers";
    const PUBLISHING: &'static [&'static str] = &["push", "commit"];
    const WRITING: &'static [&'static str] =
        &["commit", "merge", "rebase", "cherry-pick", "revert", "pull", "am", "push"];

    pub fn refusal(&self, config: &Config, lander: Option<&Actor>) -> Option<Listed> {
        self.invokes(Self::PR_MERGE)
            .map(|()| Listed::new("gh pr merge is the owner's job"))
            .or_else(|| lander.map_or_else(|| self.landing(), |_| None))
            .or_else(|| self.0.segments().filter_map(GitCall::parse).find_map(|call| call.push_refusal(config)))
    }

    fn landing(&self) -> Option<Listed> {
        let meta = |word: &String| Path::new(word).file_name().is_some_and(|name| name == Self::META);
        let queue_run = |window: &[String]| {
            window.split_first().is_some_and(|(program, rest)| meta(program) && rest == Self::QUEUE_RUN)
        };
        let lands = |segment: &&[String]| segment.windows(Self::QUEUE_RUN.len() + 1).any(queue_run);
        self.0.segments().find(lands).map(|_| Listed::new(Self::LANDING))
    }

    pub fn invokes(&self, words: &[&str]) -> Option<()> {
        self.0.segments().find(|segment| segment.windows(words.len()).any(|window| window == words)).map(|_| ())
    }

    pub fn commit_directories(&self, cwd: &Path) -> Vec<PathBuf> {
        let mut shell = WorkingDirectory::new(cwd);
        let writes = |words: &[String]| GitCall::parse(words)?.writes_commits();
        self.0.segments().filter_map(|words| shell.follow(words, writes)).collect()
    }

    pub fn publishing_directory(&self, cwd: &Path) -> Option<PathBuf> {
        let mut shell = WorkingDirectory::new(cwd);
        self.0.segments().find_map(|words| shell.follow(words, Self::publishes))
    }

    fn publishes(words: &[String]) -> Option<()> {
        let pushes =
            GitCall::parse(words).and_then(|call| Self::PUBLISHING.iter().find_map(|subcommand| call.runs(subcommand)));
        pushes.or_else(|| words.windows(Self::PR_CREATE.len()).any(|window| window == Self::PR_CREATE).then_some(()))
    }

    pub fn runs(&self, subcommand: &str) -> Option<()> {
        self.0.segments().filter_map(GitCall::parse).find_map(|call| call.runs(subcommand))
    }

    pub fn writes(&self) -> Option<()> {
        Self::WRITING.iter().find_map(|subcommand| self.runs(subcommand))
    }

    pub fn breach(&self, rules: &ShellRules) -> Option<Listed> {
        self.reason(rules).map(Listed::new)
    }

    fn reason(&self, rules: &ShellRules) -> Option<String> {
        let breach = self.0.breach().map(str::to_owned);
        breach.or_else(|| self.0.parts().find_map(|part| Self::segment_reason(rules, part)))
    }

    fn segment_reason(rules: &ShellRules, part: &Segment) -> Option<String> {
        let segment = part.words();
        let said = |word: &str, why: &str| format!("{} `{word}` {why}", Self::SHELL_RULE);
        let named = rules.forbidden_in(segment).map(|word| said(&word.to_string(), Self::USE));
        let call = rules.git_in(segment).and_then(GitCall::parse);
        let gitted =
            || call.as_ref().and_then(|call| call.breach(rules)).map(|what| said(&what.to_string(), Self::GIT_USE));
        let text = rules.nested(segment).or_else(|| call.as_ref().and_then(|call| call.nested(rules)));
        let kinds = rules.program_at(segment).map(|position| part.kinds_from(position)).unwrap_or_default();
        let built = || {
            (kinds.first() == Some(&Kind::Expanded) || (text.is_some() && kinds.contains(&Kind::Expanded)))
                .then(|| format!("{} {}", Self::SHELL_RULE, Self::EXPANDED))
        };
        let piped = || rules.stdin_shell(segment).map(|word| said(word, Self::STDIN));
        let nested = || text.as_ref().and_then(|text| text.parse::<Self>().ok()).and_then(|inner| inner.reason(rules));
        named.or_else(gitted).or_else(built).or_else(piped).or_else(nested)
    }
}

impl FromStr for ShellCommand {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(text.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::ShellCommand;
    use crate::request::project::Config;

    const WRITERS: [&str; 4] = ["git revert HEAD", "git pull", "git am x.patch", "git commit-tree t"];
    const NOT_WRITERS: [&str; 3] = ["git merge --abort", "git rebase --abort", "git merge-base a b"];
    const SESSION_WRITES: [(&str, bool); 2] = [("git pull", true), ("git merge-base a b", false)];
    const CWD: &str = "/w/good";
    const CHAINS: [(&str, &[&str]); 5] = [
        ("(cd a && git -c x=y -C b commit -m 'cd z')", &["/w/good/a/b"]),
        ("git commit -m x && git -C ../bad commit -m y", &["/w/good", "/w/good/../bad"]),
        ("git commit; cd ../bad; git commit", &["/w/good", "/w/good/../bad"]),
        ("(cd ../bad && pwd && git commit) && git commit", &["/w/good/../bad", "/w/good"]),
        ("cd $PWD/a && git -C ~/b commit && git -C $HOME commit; git -C $PWD/c commit", &["~/b", "~", "/w/good/a/c"]),
    ];

    #[test]
    fn every_commit_writer_resolves_its_own_directory_and_aborts_write_nothing() {
        let home = std::env::var("HOME").unwrap();
        let directories = |command: &str| command.parse::<ShellCommand>().unwrap().commit_directories(Path::new(CWD));
        let expected =
            |paths: &[&str]| paths.iter().map(|path| PathBuf::from(path.replacen('~', &home, 1))).collect::<Vec<_>>();
        let writes =
            |(command, writes): &(&str, bool)| command.parse::<ShellCommand>().unwrap().writes().is_some() == *writes;
        assert!(WRITERS.iter().all(|command| directories(command).len() == 1) && SESSION_WRITES.iter().all(writes));
        assert!(NOT_WRITERS.iter().all(|command| directories(command).is_empty()));
        assert!(CHAINS.iter().all(|(command, paths)| directories(command) == expected(paths)));
    }

    const CONFIG: &str = "protected = [\"main\"]\n";
    const MERGES: [&str; 4] = [
        "gh pr merge 1",
        "cd x && gh pr merge 1 --squash",
        "meta queue run 1",
        "cd x && target/release/meta queue run 1 2",
    ];
    const PUSHES: [&str; 8] = [
        "git push --forc origin feature",
        "/usr/bin/git push --force origin feature",
        "git push --force origin feature",
        "git push -uf origin feature",
        "git push origin +feature",
        "git -C x push origin HEAD:refs/heads/main",
        "git push --force-with-lease origin main",
        "git --no-pager push origin main",
    ];
    const ALLOWED: [&str; 7] = [
        "git push --force-w origin feature",
        "git push --force-if-includes origin feature",
        "git push -u origin feature",
        "git push --force-with-lease origin feature",
        "git commit -m 'git push origin main'",
        "gh pr view 1",
        "meta status --flags",
    ];
    const PUSH_REASON: &str = "git push";
    const LOG_CONFIG: &str = "protected = [\"main\", \"meta/log\", \"meta/logs\"]\nlog_branches = [\"meta/log\"]\n";
    const LOG_PUSH: &str = "git push origin meta/log";
    const ONLY_META: &str = "only `meta` writes it";
    const FORCED: &str = "git push --force origin feature";
    const LEASE: &str = "lease";
    const NEAR_LOG: &str = "git push origin meta/logs";
    const OPEN_PR: &str = "open a PR";
    const LEAD: &str = "claude-lead";
    const LANDS: &str = "meta queue run 1";

    #[test]
    fn merges_force_pushes_and_protected_pushes_are_refused_and_nothing_else() {
        let config: Config = toml::from_str(CONFIG).unwrap();
        let refusal =
            |command: &str| command.parse::<ShellCommand>().unwrap().refusal(&config, None).map(|why| why.to_string());
        assert!(MERGES.iter().all(|command| refusal(command).is_some()));
        assert!(PUSHES.iter().all(|command| refusal(command).is_some_and(|why| why.starts_with(PUSH_REASON))));
        assert!(ALLOWED.iter().all(|command| refusal(command).is_none()));
    }

    #[test]
    fn a_log_push_names_meta_and_a_force_push_names_no_lease() {
        let config: Config = toml::from_str(LOG_CONFIG).unwrap();
        let lead = LEAD.parse().unwrap();
        let refused = |command: &str| command.parse::<ShellCommand>().unwrap().refusal(&config, Some(&lead));
        let refusal = |command: &str| refused(command).unwrap().to_string();
        assert!(refusal(LOG_PUSH).ends_with(ONLY_META) && refusal(NEAR_LOG).ends_with(OPEN_PR));
        assert!(!refusal(FORCED).contains(LEASE) && refused(LANDS).is_none());
    }
}
