use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantArray};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, EnumString, VariantArray, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Verb {
    Concern,
    Issue,
    Rate,
    Move,
    Open,
    Wait,
    Resume,
    Link,
    Claim,
    Worktrees,
    Gc,
    Release,
    Done,
    Reopen,
    Retire,
    Correct,
    List,
    Status,
    Hook,
    Heartbeat,
    Monitor,
    CiWatch,
    CiWait,
    CiLog,
    Trace,
    Other,
}
