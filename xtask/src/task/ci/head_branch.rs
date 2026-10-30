use std::collections::BTreeSet;
use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use meta_git::RefName;
use meta_schema::{Id, Task};
use task::BranchName;

use crate::task::ci::Unready;
use crate::task::ci::untracked::Untracked;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadBranch {
    name: BranchName,
    task: Option<Id<Task>>,
}

impl HeadBranch {
    const REMOTE_HEAD: &str = "HEAD";
    const REMOTE_PARTS_LEN: usize = 4;

    pub fn only(listed: &str) -> Result<HeadBranch, Unready> {
        let names: BTreeSet<&str> = listed.lines().filter(|name| *name != Self::REMOTE_HEAD).collect();
        let only = names.first().filter(|_| names.len() == 1).ok_or(Unready::Unnamed)?;
        let Ok(head) = only.parse();
        Ok(head)
    }

    pub fn listed(tips: &[RefName]) -> String {
        let short =
            |tip: &RefName| tip.to_string().splitn(Self::REMOTE_PARTS_LEN, '/').last().unwrap_or_default().to_owned();
        tips.iter().map(short).collect::<Vec<_>>().join("\n")
    }

    pub fn task(&self) -> Result<Id<Task>, Untracked> {
        self.task.ok_or_else(|| Untracked::NotTask(self.name.clone()))
    }
}

impl fmt::Display for HeadBranch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}

impl FromStr for HeadBranch {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<HeadBranch, Infallible> {
        let name = text.trim();
        let cut = name.match_indices(['-', '/']).nth(1).map_or(name.len(), |(at, _)| at);
        let Ok(branch) = name.parse();
        Ok(HeadBranch { name: branch, task: name.get(..cut).and_then(|id| id.parse().ok()) })
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use meta_golden::GoldenFailure;
    use meta_run::GitArgv;

    use super::HeadBranch;
    use crate::task::ci::Unready;
    use crate::task::revision::{Commit, GitReads};

    const VIEWED: [GitArgv; 4] = [GitArgv::Porcelain, GitArgv::PushedHead, GitArgv::Head, GitArgv::PushedTips];
    const LISTED: [&str; 4] = ["HEAD\nmain\n", "main\nsecond\n", "HEAD\n", ""];
    const MAIN: &str = "main";

    impl HeadBranch {
        fn view(call: GitArgv, out: &str) -> Box<dyn Debug> {
            match call {
                GitArgv::Head => Box::new(out.parse::<Commit>()),
                GitArgv::PushedTips => Box::new(HeadBranch::only(out)),
                _ => Box::new(out.is_empty()),
            }
        }
    }

    #[test]
    fn the_judged_branch_is_the_only_one_pushed_at_head() {
        let named = LISTED.map(HeadBranch::only);
        let unnamed = Err(Unready::Unnamed);
        assert_eq!(named, [Ok(MAIN.parse().unwrap()), unnamed.clone(), unnamed.clone(), unnamed]);
    }

    #[test]
    fn every_recorded_gate_read_parses_to_its_golden_view() -> Result<(), GoldenFailure> {
        assert!(GitReads::all_viewed(&VIEWED, HeadBranch::view)?);
        Ok(())
    }
}
