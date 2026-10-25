mod git_call;

use std::fmt;

use serde::Deserialize;

use crate::request::output::Listed;
use crate::request::project::Project;
use git_call::GitCall;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct ShellCommand(String);

impl ShellCommand {
    const SEPARATORS: &'static [char] = &[';', '&', '|', '\n', '(', ')'];
    const QUOTES: &'static [char] = &['\'', '"'];
    const PR_MERGE: &'static [&'static str] = &["gh", "pr", "merge"];

    pub fn refusal(&self, project: &Project) -> Option<Listed> {
        self.segments()
            .find(|words| words.windows(Self::PR_MERGE.len()).any(|window| window == Self::PR_MERGE))
            .map(|_| Listed::new("gh pr merge is the owner's job"))
            .or_else(|| {
                self.segments()
                    .filter_map(|segment| GitCall::parse(&segment))
                    .find_map(|call| call.push_refusal(project))
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
