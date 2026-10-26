mod git_call;

use std::fmt;

use serde::Deserialize;

use crate::request::output::Listed;
use crate::request::project::Config;
use git_call::GitCall;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct ShellCommand(String);

impl ShellCommand {
    const SEPARATORS: &'static [char] = &[';', '&', '|', '\n', '(', ')'];
    const QUOTES: &'static [char] = &['\'', '"'];
    const PR_MERGE: &'static [&'static str] = &["gh", "pr", "merge"];

    pub fn refusal(&self, config: &Config) -> Option<Listed> {
        self.segments()
            .find(|words| words.windows(Self::PR_MERGE.len()).any(|window| window == Self::PR_MERGE))
            .map(|_| Listed::new("gh pr merge is the owner's job"))
            .or_else(|| {
                self.segments()
                    .filter_map(|segment| GitCall::parse(&segment))
                    .find_map(|call| call.push_refusal(config))
            })
    }

    pub fn commits(&self) -> Option<&Self> {
        self.segments().filter_map(|segment| GitCall::parse(&segment)).find_map(|call| call.commit()).map(|()| self)
    }

    fn segments(&self) -> impl Iterator<Item = Vec<String>> + '_ {
        self.0.split(Self::SEPARATORS).map(|segment| {
            segment
                .split_whitespace()
                .map(|word| word.trim_matches(Self::QUOTES).to_owned())
                .filter(|word| !word.is_empty())
                .collect()
        })
    }
}

impl fmt::Display for ShellCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ShellCommand;
    use crate::request::project::Config;

    const CONFIG: &str = "protected = [\"main\"]\n";
    const BLOCKED: [&str; 7] = [
        "gh pr merge 1",
        "cd x && gh pr merge 1 --squash",
        "git push --force origin feature",
        "git push -uf origin feature",
        "git push origin +feature",
        "git -C x push origin HEAD:refs/heads/main",
        "git push --force-with-lease origin main",
    ];
    const ALLOWED: [&str; 4] = [
        "git push -u origin feature",
        "git push --force-with-lease origin feature",
        "git commit -m 'git push origin main'",
        "gh pr view 1",
    ];

    #[test]
    fn merges_force_pushes_and_protected_pushes_are_refused_and_nothing_else() {
        let config: Config = toml::from_str(CONFIG).unwrap();
        let refused = |command: &str| ShellCommand(command.to_owned()).refusal(&config).is_some();
        assert!(BLOCKED.iter().all(|command| refused(command)));
        assert!(!ALLOWED.iter().any(|command| refused(command)));
    }
}
