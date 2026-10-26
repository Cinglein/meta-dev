use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[repr(u8)]
pub enum MutantsArgument {
    #[strum(serialize = "--in-diff")]
    InDiff,
    #[strum(serialize = "--in-place")]
    InPlace,
    #[strum(serialize = "--output")]
    Output,
    #[strum(serialize = "--shard")]
    Shard,
    #[strum(serialize = "--list")]
    List,
    #[strum(serialize = "--json")]
    Json,
    #[strum(serialize = "--jobserver-tasks")]
    JobserverTasks,
    #[strum(serialize = "--test-package")]
    TestPackage,
    #[strum(serialize = "--test-workspace=true")]
    TestWorkspaceOn,
    #[strum(serialize = "--test-workspace=false")]
    TestWorkspaceOff,
    #[strum(serialize = "--baseline=skip")]
    BaselineSkip,
    #[strum(serialize = "--timeout")]
    Timeout,
}
