use crate::env_policy::EnvPolicy;
use crate::tool::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PullArgv {
    DefaultBranch,
    StrictRules,
    UpdateBranch,
    SquashMerge,
}

impl Tool for PullArgv {
    const PROGRAM: &'static str = "gh";
    const POLICY: EnvPolicy = EnvPolicy::Inherit;

    fn words(self) -> &'static [&'static str] {
        match self {
            Self::DefaultBranch => &["api", "repos/{owner}/{repo}", "--jq", ".default_branch"],
            Self::StrictRules => &[
                "api",
                "--jq",
                "[.[] | select(.type == \"required_status_checks\") | .parameters.strict_required_status_checks_policy] | any",
            ],
            Self::UpdateBranch => &["pr", "update-branch"],
            Self::SquashMerge => &["pr", "merge", "--squash", "--match-head-commit"],
        }
    }
}
