use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[repr(u8)]
pub enum TestArgument {
    #[strum(serialize = "--list")]
    List,
    #[strum(serialize = "--format")]
    Format,
    #[strum(serialize = "terse")]
    Terse,
    #[strum(serialize = "--exact")]
    Exact,
    #[strum(serialize = "--quiet")]
    Quiet,
}
