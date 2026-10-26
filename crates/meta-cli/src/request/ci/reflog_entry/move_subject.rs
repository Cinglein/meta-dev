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
