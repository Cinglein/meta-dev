use std::fmt;
use std::str::FromStr;

use meta_log::{GitLog, LogStore, Sha, Snapshot};

use crate::task::failure::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogSource {
    Origin,
    Prefetched(Sha),
    Unfetched,
}

impl LogSource {
    pub const FLAG: &str = "--log=";
    const ORIGIN: &str = "origin";
    const UNFETCHED: &str = "unfetched";
    const TRACKING: &str = "refs/remotes/origin/meta/log";

    fn tracking() -> Sha {
        let Ok(tracking) = Self::TRACKING.parse();
        tracking
    }

    pub fn snapshot(&self, log: &GitLog) -> Result<Snapshot, Failure> {
        match self {
            LogSource::Origin => log.read().map_err(Failure::StandingLog),
            LogSource::Prefetched(_) => log.snapshot_at(&Self::tracking()).map_err(Failure::StandingLog),
            LogSource::Unfetched => log.snapshot_at(&Self::tracking()).map_err(|_| Failure::Unfetched),
        }
    }
}

impl FromStr for LogSource {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<LogSource, Self::Err> {
        Ok(match text {
            Self::ORIGIN => LogSource::Origin,
            Self::UNFETCHED => LogSource::Unfetched,
            tip => LogSource::Prefetched(tip.parse()?),
        })
    }
}

impl fmt::Display for LogSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogSource::Origin => write!(formatter, "{}{}", Self::FLAG, Self::ORIGIN),
            LogSource::Prefetched(tip) => write!(formatter, "{}{tip}", Self::FLAG),
            LogSource::Unfetched => write!(formatter, "{}{}", Self::FLAG, Self::UNFETCHED),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use super::LogSource;
    use crate::task::Task;
    use crate::task::coverage::Coverage;

    const FLAGS: [&str; 3] = ["--log=origin", "--log=unfetched", "--log=0a1b2c"];

    #[test]
    fn every_source_round_trips_through_the_ci_flag_and_other_tasks_refuse_it() {
        let parsed =
            |task, flag: &str| Coverage::from_flags(task, iter::once(flag.into())).map(|cov| cov.log().clone());
        let sources = FLAGS.map(|flag| parsed(Task::Ci, flag).unwrap());
        assert!(sources.iter().zip(FLAGS).all(|(source, flag)| source.to_string() == flag));
        let [origin, unfetched, _] = &sources;
        assert!(*origin == LogSource::Origin && *unfetched == LogSource::Unfetched);
        assert!(parsed(Task::Test, &origin.to_string()).is_err());
    }
}
