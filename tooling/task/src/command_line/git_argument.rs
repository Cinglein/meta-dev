use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[repr(u8)]
pub enum GitArgument {
    #[strum(serialize = "diff")]
    Diff,
    #[strum(serialize = "--name-only")]
    NameOnly,
    #[strum(serialize = "--merge-base")]
    MergeBase,
}
