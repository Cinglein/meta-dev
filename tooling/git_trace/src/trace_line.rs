mod sid;

use serde::Deserialize;

use super::starter::Starter;
use sid::Sid;

#[derive(Debug, Deserialize)]
pub(crate) struct TraceLine {
    sid: Sid,
}

impl TraceLine {
    pub(crate) fn starter(trace: &str) -> Starter {
        let first = trace.lines().next().and_then(|line| serde_json::from_str::<TraceLine>(line).ok());
        first.and_then(|line| line.sid.invocation()).map_or(Starter::Tests, Starter::Meta)
    }
}
