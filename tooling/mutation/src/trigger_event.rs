use std::env;

use strum::EnumString;

use task::FullRun;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "snake_case")]
#[repr(u8)]
pub enum TriggerEvent {
    Push,
    Schedule,
    WorkflowDispatch,
    PullRequest,
    Local,
}

impl TriggerEvent {
    const VARIABLE: &str = "GITHUB_EVENT_NAME";

    #[must_use]
    pub fn current() -> Option<TriggerEvent> {
        Self::named(env::var(Self::VARIABLE).ok())
    }

    #[must_use]
    pub fn named(name: Option<String>) -> Option<TriggerEvent> {
        name.map_or(Some(TriggerEvent::Local), |name| name.parse().ok())
    }

    #[must_use]
    pub const fn full_run(self) -> Option<FullRun> {
        match self {
            TriggerEvent::Push => Some(FullRun::EveryMainPush),
            TriggerEvent::Schedule => Some(FullRun::Weekly),
            TriggerEvent::WorkflowDispatch | TriggerEvent::Local => Some(FullRun::OnDemand),
            TriggerEvent::PullRequest => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FullRun, TriggerEvent};
    const NAMES: [&str; 4] = ["push", "schedule", "workflow_dispatch", "pull_request"];
    const RUNS: [Option<FullRun>; 4] =
        [Some(FullRun::EveryMainPush), Some(FullRun::Weekly), Some(FullRun::OnDemand), None];

    #[test]
    fn each_github_event_names_the_full_run_cadence_it_stands_for() {
        let runs = NAMES.map(|name| name.parse::<TriggerEvent>().ok().and_then(TriggerEvent::full_run));
        assert_eq!(runs, RUNS);
        assert!("release".parse::<TriggerEvent>().is_err());
        assert_eq!([None, Some(String::from("release"))].map(TriggerEvent::named), [Some(TriggerEvent::Local), None]);
    }
}
