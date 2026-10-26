use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[repr(u8)]
pub enum CargoArgument {
    #[strum(serialize = "fmt")]
    Fmt,
    #[strum(serialize = "clippy")]
    Clippy,
    #[strum(serialize = "test")]
    Test,
    #[strum(serialize = "metadata")]
    Metadata,
    #[strum(serialize = "--all")]
    All,
    #[strum(serialize = "--check")]
    Check,
    #[strum(serialize = "--workspace")]
    Workspace,
    #[strum(serialize = "--package")]
    Package,
    #[strum(serialize = "--all-targets")]
    AllTargets,
    #[strum(serialize = "--all-features")]
    AllFeatures,
    #[strum(serialize = "--no-run")]
    NoRun,
    #[strum(serialize = "--color")]
    Color,
    #[strum(serialize = "never")]
    Never,
    #[strum(serialize = "--jobs")]
    Jobs,
    #[strum(serialize = "--format-version=1")]
    FormatVersionOne,
    #[strum(serialize = "--no-deps")]
    NoDeps,
    #[strum(serialize = "--")]
    ToolArguments,
    #[strum(serialize = "-D")]
    Deny,
    #[strum(serialize = "warnings")]
    Warnings,
}
