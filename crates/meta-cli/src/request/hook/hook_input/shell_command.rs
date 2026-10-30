mod git_call;
mod tokens;
mod working_directory;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::request::output::Listed;
use crate::request::project::Config;
use crate::request::scale_policy::ShellRules;
use git_call::GitCall;
use tokens::Tokens;
use working_directory::WorkingDirectory;

#[derive(Debug)]
pub struct ShellCommand(Tokens);

impl ShellCommand {
    const SHELL_RULE: &'static str = "shell rule:";
    const USE: &'static str =
        "is not allowed; use the Write or Edit tool, `git restore`, or `git rm` inside a code change";
    const STDIN: &'static str = "reads its commands from stdin; pass them with -c so they can be checked";
    const PR_MERGE: &'static [&'static str] = &["gh", "pr", "merge"];
    const PR_CREATE: &'static [&'static str] = &["gh", "pr", "create"];
    const PUBLISHING: &'static [&'static str] = &["push", "commit"];
    const WRITING: &'static [&'static str] =
        &["commit", "merge", "rebase", "cherry-pick", "revert", "pull", "am", "push"];

    pub fn refusal(&self, config: &Config) -> Option<Listed> {
        self.invokes(Self::PR_MERGE)
            .map(|()| Listed::new("gh pr merge is the owner's job"))
            .or_else(|| self.segments().filter_map(GitCall::parse).find_map(|call| call.push_refusal(config)))
    }

    pub fn invokes(&self, words: &[&str]) -> Option<()> {
        self.segments().find(|segment| segment.windows(words.len()).any(|window| window == words)).map(|_| ())
    }

    pub fn commit_directories(&self, cwd: &Path) -> Vec<PathBuf> {
        let mut shell = WorkingDirectory::new(cwd);
        let writes = |words: &[String]| GitCall::parse(words)?.writes_commits();
        self.segments().filter_map(|words| shell.follow(words, writes)).collect()
    }

    pub fn publishing_directory(&self, cwd: &Path) -> Option<PathBuf> {
        let mut shell = WorkingDirectory::new(cwd);
        self.segments().find_map(|words| shell.follow(words, Self::publishes))
    }

    fn publishes(words: &[String]) -> Option<()> {
        let pushes =
            GitCall::parse(words).and_then(|call| Self::PUBLISHING.iter().find_map(|subcommand| call.runs(subcommand)));
        pushes.or_else(|| words.windows(Self::PR_CREATE.len()).any(|window| window == Self::PR_CREATE).then_some(()))
    }

    pub fn runs(&self, subcommand: &str) -> Option<()> {
        self.segments().filter_map(GitCall::parse).find_map(|call| call.runs(subcommand))
    }

    pub fn writes(&self) -> Option<()> {
        Self::WRITING.iter().find_map(|subcommand| self.runs(subcommand))
    }

    fn segments(&self) -> impl Iterator<Item = &[String]> {
        self.0.segments()
    }

    pub fn breach(&self, rules: &ShellRules) -> Option<Listed> {
        self.reason(rules).map(Listed::new)
    }

    fn reason(&self, rules: &ShellRules) -> Option<String> {
        let program = |segment: &[String]| {
            let said = |word: &str, why: &str| format!("{} `{word}` {why}", Self::SHELL_RULE);
            let named = rules.forbidden_in(segment).map(|word| said(word, Self::USE));
            let piped = || rules.stdin_shell(segment).map(|word| said(word, Self::STDIN));
            named
                .or_else(piped)
                .or_else(|| rules.nested(segment).and_then(|text| Self(Tokens::scan(&text)).reason(rules)))
        };
        self.0.breach().map(str::to_owned).or_else(|| self.segments().find_map(program))
    }
}

impl FromStr for ShellCommand {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(Tokens::scan(text)))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::ShellCommand;
    use crate::request::project::Config;
    use crate::request::scale_policy::ScalePolicy;

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

    const VERDICTS: [(&str, bool); 56] = [
        ("rm -rf x", true),
        ("cd a && mv b c", true),
        ("sed -i s/a/b/ f", true),
        ("git diff | sed -n 1p", true),
        ("awk '{print}' f", true),
        ("python3 -c 1", true),
        ("nice -n 19 perl -e 1", true),
        ("FOO=1 /usr/bin/sed p", true),
        ("cat <<EOF\nx\nEOF", true),
        ("cat > f", true),
        ("echo x >> f", true),
        ("ls 2>/dev/null", true),
        ("ls &>f", true),
        ("git commit -m 'remove sed > rm'", false),
        ("git commit -m \"rm and sed\" && git rm x", false),
        ("ls crates/rm/src/sed.rs | grep rm", false),
        ("cargo build 2>&1 | tail -3", false),
        ("cat f < in; grep x <<< y", false),
        ("echo a\\>b; FOO=rm env ls", false),
        ("git diff | wc -l", false),
        ("timeout 5s sed p f", true),
        ("timeout -k 1 5 nice -n 2 awk 1", true),
        ("cat <<-'EOF'\nx\nEOF", true),
        ("echo $((1 << 2))", false),
        ("true; xargs ls", false),
        ("git commit -m \"fix; rm stuff\"", false),
        ("echo \"a | sed\" && git commit -m 'fix (sed) bug'", false),
        ("git commit -m \"title\n\nrm the thing\nsed it\npython x\"", false),
        ("command -v sed; echo a # > b", false),
        ("diff <(ls) <(ls -a); tee >(cat)", false),
        ("bash -c 'sed -i p f'", true),
        ("sh -c \"cd a && rm x\"", true),
        ("eval rm x", true),
        ("xargs -n 1 sh -lc 'mv a b'", true),
        ("echo $(sed p f)", true),
        ("echo `rm x`", true),
        ("echo \"$(rm x)\"", true),
        ("echo \"$(echo a > f)\"", true),
        ("env -u X sed p", true),
        ("sudo -u X rm x", true),
        ("timeout -s KILL 5 sed p", true),
        ("timeout --signal KILL -k 1 5 sed p", true),
        ("cd a; (sed p f)", true),
        ("sudo -n rm x", true),
        ("sudo -E rm x", true),
        ("sudo -S rm x", true),
        ("sudo -k rm x", true),
        ("if rm x; then :; fi", true),
        ("while sed p f; do :; done", true),
        ("echo rm x | bash", true),
        ("bash <<< 'rm x'", true),
        ("bash script.sh; source f", false),
        ("gh pr create --body \"Fix (a; b) > c\n\n- rm x; sed\"", false),
        ("cargo test -- --nocapture 2>&1 | tail", false),
        ("jq '.[] | select(.a > 1)' f", false),
        ("RUSTFLAGS=\"-D warnings\" nice -n 19 cargo clippy", false),
    ];

    #[test]
    fn a_shell_command_running_a_forbidden_program_heredoc_or_file_redirect_is_refused_and_nothing_else() {
        let rules = ScalePolicy::shell_in(None).unwrap();
        let refused = |command: &str| command.parse::<ShellCommand>().unwrap().breach(&rules).is_some();
        assert!(VERDICTS.iter().all(|(command, verdict)| refused(command) == *verdict));
    }

    #[test]
    fn a_log_push_names_meta_and_a_force_push_names_no_lease() {
        let config: Config = toml::from_str(LOG_CONFIG).unwrap();
        let refusal = |command: &str| command.parse::<ShellCommand>().unwrap().refusal(&config).unwrap().to_string();
        assert!(refusal(LOG_PUSH).ends_with(ONLY_META) && refusal(NEAR_LOG).ends_with(OPEN_PR));
        assert!(!refusal(FORCED).contains(LEASE));
    }
}
