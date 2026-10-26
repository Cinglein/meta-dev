use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveSubject(String);

impl MoveSubject {
    const ACTION_END: &'static str = ": ";
    const CREATING: [&'static str; 6] =
        ["commit", "commit (amend)", "commit (merge)", "commit (initial)", "cherry-pick", "revert"];
    const REPLAYERS: [&'static str; 2] = ["rebase", "pull"];
    const PICKS: [&'static str; 5] = ["(pick)", "(reword)", "(edit)", "(squash)", "(fixup)"];
    const MERGERS: [&'static str; 2] = ["merge ", "pull"];
    const MERGE_MADE: &'static str = "Merge made by the '";

    pub fn creates(&self) -> bool {
        self.0.split_once(Self::ACTION_END).is_some_and(|(action, rest)| {
            Self::CREATING.contains(&action)
                || (Self::REPLAYERS.iter().any(|replayer| action.starts_with(replayer))
                    && Self::PICKS.iter().any(|pick| action.ends_with(pick)))
                || (Self::MERGERS.iter().any(|merger| action.starts_with(merger)) && rest.starts_with(Self::MERGE_MADE))
        })
    }
}

impl FromStr for MoveSubject {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(text.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::MoveSubject;

    const CREATING: [&str; 17] = [
        "commit: x",
        "commit (initial): x",
        "commit (amend): x",
        "commit (merge): x",
        "commit (cherry-pick): x",
        "cherry-pick: x",
        "revert: Revert \"x\"",
        "rebase (pick): x",
        "rebase (reword): x",
        "rebase (edit): x",
        "rebase (squash): x",
        "rebase (fixup): x",
        "rebase (continue): x",
        "rebase -i (pick): x",
        "am: x",
        "merge main: Merge made by the 'ort' strategy.",
        "pull: Merge made by the 'ort' strategy.",
    ];
    const MOVING: [&str; 9] = [
        "checkout: moving from main to x",
        "reset: moving to HEAD",
        "pull: Fast-forward",
        "merge x: Fast-forward",
        "rebase (start): checkout main",
        "rebase (finish): returning to refs/heads/x",
        "rebase (abort): returning to refs/heads/x",
        "clone: from /x",
        "branch: Created from HEAD",
    ];

    #[test]
    fn every_commit_creating_subject_git_writes_creates_and_no_move_does() {
        let creates = |subject: &&str| subject.parse::<MoveSubject>().is_ok_and(|subject| subject.creates());
        assert!(CREATING.iter().all(creates) && !MOVING.iter().any(creates));
    }
}
