use strum::{Display, EnumIter, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumString, EnumIter)]
pub enum CommitAction {
    #[strum(serialize = "commit")]
    Commit,
    #[strum(serialize = "commit (initial)")]
    CommitInitial,
    #[strum(serialize = "commit (amend)")]
    CommitAmend,
    #[strum(serialize = "commit (merge)")]
    CommitMerge,
    #[strum(serialize = "commit (cherry-pick)")]
    CommitCherryPick,
    #[strum(serialize = "cherry-pick")]
    CherryPick,
    #[strum(serialize = "revert")]
    Revert,
    #[strum(serialize = "rebase (pick)")]
    RebasePick,
    #[strum(serialize = "rebase (reword)")]
    RebaseReword,
    #[strum(serialize = "rebase (edit)")]
    RebaseEdit,
    #[strum(serialize = "rebase (squash)")]
    RebaseSquash,
    #[strum(serialize = "rebase (fixup)")]
    RebaseFixup,
    #[strum(serialize = "rebase (continue)")]
    RebaseContinue,
    #[strum(serialize = "am")]
    Am,
    #[strum(serialize = "merge (strategy)")]
    MergeByStrategy,
}
