mod git_call;

use std::str::FromStr;

use crate::request::output::Listed;
use crate::request::project::Project;
use git_call::GitCall;

#[derive(Debug)]
pub struct ShellCommand(Vec<Vec<String>>);

impl ShellCommand {
    const SEPARATORS: &'static [char] = &[';', '&', '|', '\n', '(', ')'];
    const QUOTES: &'static [char] = &['\'', '"'];
    const PR_MERGE: &'static [&'static str] = &["gh", "pr", "merge"];

    pub fn refusal(&self, project: &Project) -> Option<Listed> {
        self.segments()
            .find(|words| words.windows(Self::PR_MERGE.len()).any(|window| window == Self::PR_MERGE))
            .map(|_| Listed::new("gh pr merge is the owner's job"))
            .or_else(|| self.segments().filter_map(GitCall::parse).find_map(|call| call.push_refusal(project)))
    }

    pub fn commits(&self) -> Option<()> {
        self.segments().filter_map(GitCall::parse).find_map(|call| call.commit())
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
