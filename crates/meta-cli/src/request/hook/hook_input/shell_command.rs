mod git_call;

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
    use super::ShellCommand;
    use crate::request::project::Config;

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
