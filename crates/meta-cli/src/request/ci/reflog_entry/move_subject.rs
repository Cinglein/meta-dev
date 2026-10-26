mod commit_action;

use std::str::FromStr;

use commit_action::CommitAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveSubject(String);

impl MoveSubject {
    const ACTION_END: &'static str = ": ";
    const REPLAYERS: [&'static str; 2] = ["rebase", "pull"];
    const REPLAYED: &'static str = "rebase";
    const MERGERS: [&'static str; 2] = ["merge ", "pull"];
    const MERGE_MADE: &'static str = "Merge made by the '";

    pub fn creates(&self) -> bool {
        self.action().is_some()
    }

    fn action(&self) -> Option<CommitAction> {
        let (action, rest) = self.0.split_once(Self::ACTION_END)?;
        let kind = Self::REPLAYERS
            .iter()
            .any(|replayer| action.starts_with(replayer))
            .then(|| action.rsplit_once(' '))
            .flatten();
        let named = kind.map_or_else(|| action.to_owned(), |(_, kind)| [Self::REPLAYED, kind].join(" "));
        (Self::MERGERS.iter().any(|merger| action.starts_with(merger)) && rest.starts_with(Self::MERGE_MADE))
            .then_some(CommitAction::MergeByStrategy)
            .or_else(|| named.parse().ok())
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
    use strum::IntoEnumIterator as _;

    use super::{CommitAction, MoveSubject};

    const CREATING: [&str; 19] = [
        "rebase (merge): x",
        "merge side: Zusammenführung durch die Strategie 'ort'.",
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
        "cherry-pick: fast-forward",
        "rebase (reset): 'branch-point'",
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
        let actions: Vec<CommitAction> =
            CREATING.iter().filter_map(|subject| subject.parse::<MoveSubject>().ok()?.action()).collect();
        assert!(CommitAction::iter().all(|action| actions.contains(&action)));
    }
}
