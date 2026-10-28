mod commit_action;

use std::str::FromStr;

pub use commit_action::CommitAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveSubject(String);

impl MoveSubject {
    const ACTION_END: &'static str = ": ";
    const FAST_FORWARD: &'static str = "fast-forward";
    const REPLAYERS: [&'static str; 2] = ["rebase", "pull"];
    const REPLAYED: &'static str = "rebase";
    const KIND_OPEN: char = '(';
    const MERGERS: [&'static str; 2] = ["merge ", "pull"];

    pub fn action(&self) -> Option<CommitAction> {
        let (action, _) = self.0.split_once(Self::ACTION_END).filter(|(_, rest)| *rest != Self::FAST_FORWARD)?;
        let kind = Self::REPLAYERS
            .iter()
            .any(|replayer| action.starts_with(replayer))
            .then(|| action.rsplit_once(' ').filter(|(_, kind)| kind.starts_with(Self::KIND_OPEN)))
            .flatten();
        let named = kind.map_or_else(|| action.to_owned(), |(_, kind)| [Self::REPLAYED, kind].join(" "));
        (kind.is_none() && Self::MERGERS.iter().any(|merger| action.starts_with(merger)))
            .then_some(CommitAction::Merge)
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
    const MOVING: [&str; 6] = [
        "reset: moving to HEAD",
        "cherry-pick: fast-forward",
        "rebase (reset): 'branch-point'",
        "rebase (abort): returning to refs/heads/x",
        "clone: from /x",
        "branch: Created from HEAD",
    ];

    #[test]
    fn every_commit_creating_subject_git_writes_creates_and_no_move_does() {
        let creates = |subject: &&str| subject.parse::<MoveSubject>().is_ok_and(|subject| subject.action().is_some());
        assert!(CREATING.iter().all(creates) && !MOVING.iter().any(creates));
        let actions: Vec<CommitAction> =
            CREATING.iter().filter_map(|subject| subject.parse::<MoveSubject>().ok()?.action()).collect();
        assert!(CommitAction::iter().all(|action| actions.contains(&action)));
    }
}
