mod git_work;
mod log_window;
mod millis;
mod poll_schedule;
mod program_name;
mod stop_limit;

use std::time::{Duration, Instant};

use meta_pulls::Patience;
use meta_run::Budget;
use serde::Deserialize;

use super::Config;
use crate::request::cli_failure::{CliFailure, Setup};
pub use git_work::GitWork;
pub use log_window::LogWindow;
pub use millis::Millis;
use poll_schedule::PollSchedule;
pub use program_name::ProgramName;
use stop_limit::StopLimit;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CiSettings {
    program: ProgramName,
    poll_millis: Millis,
    timeout_millis: Millis,
    appear_millis: Millis,
    remote_millis: Millis,
    git_millis: Millis,
    claim_millis: Millis,
    gh_millis: Millis,
    stop_millis: StopLimit,
    queue_poll_millis: Millis,
    queue_checks_millis: Millis,
    #[serde(flatten)]
    poll_schedule: PollSchedule,
    #[serde(flatten)]
    log_window: LogWindow,
}

impl CiSettings {
    pub const DEFAULTS: &'static str = include_str!("ci_settings/ci_defaults.toml");
    pub const ROUTINE: GitWork = GitWork::Routine;
    pub const CLAIM: GitWork = GitWork::Claim;
    const UNDER_FLOOR: &'static str =
        "meta.toml [ci] queue_poll_millis is under tooling/policy.toml [millis] queue_pace";

    pub fn program(&self) -> &ProgramName {
        &self.program
    }

    pub fn poll_interval(&self) -> Millis {
        self.poll_millis
    }

    pub fn poll_schedule(&self) -> PollSchedule {
        self.poll_schedule
    }

    pub fn run_overdue(&self, waited: Duration) -> Option<()> {
        (waited >= self.appear_millis.duration()).then_some(())
    }

    pub fn remaining(&self, waited: Duration) -> Option<Duration> {
        self.timeout_millis.duration().checked_sub(waited).filter(|left| !left.is_zero())
    }

    pub fn remote_deadline(&self) -> Instant {
        Instant::now() + self.remote_millis.duration()
    }

    pub fn budget(&self, work: GitWork) -> Budget {
        Budget::new(work.limit(self.git_millis, self.claim_millis).duration())
    }

    pub fn gh_limit(&self) -> Duration {
        self.gh_millis.duration().min(self.timeout_millis.duration())
    }

    pub fn stop_limit(&self) -> StopLimit {
        self.stop_millis
    }

    pub fn queue_pace(&self, floor: Duration) -> Result<Duration, CliFailure> {
        let pace = self.queue_poll_millis.duration();
        (pace >= floor).then_some(pace).ok_or_else(|| CliFailure::raised::<Setup>(Self::UNDER_FLOOR))
    }

    pub fn queue_turns(&self, pace: Duration) -> impl Iterator<Item = ()> {
        let polls = self.queue_checks_millis.duration().as_millis() / pace.as_millis().max(1);
        std::iter::repeat_n((), usize::try_from(polls).unwrap_or(usize::MAX).saturating_add(1))
    }

    pub fn queue_patience(&self, waited: Duration) -> Patience {
        Patience::new(waited, self.queue_checks_millis.duration())
    }

    pub fn log_window(&self) -> &LogWindow {
        &self.log_window
    }
}

impl TryFrom<&Config> for CiSettings {
    type Error = CliFailure;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let mut merged: toml::Table = toml::from_str(Self::DEFAULTS).unwrap_or_default();
        merged.extend(config.ci.clone());
        toml::Value::Table(merged)
            .try_into()
            .map_err(|failure| CliFailure::raised::<Setup>(format!("meta.toml [ci]: {failure}")))
    }
}

#[cfg(test)]
mod tests {
    use super::{CiSettings, Duration};
    use crate::request::project::config::Config;

    const CONFIGURED: &str = "[ci]\npoll_millis = 7\nappear_millis = 9";
    const STOP_LIMITS: [&str; 2] = ["[ci]\nstop_millis = 6999", "[ci]\nstop_millis = 7000"];
    const REFUSED: &str = "stop_millis must be under 7000 ms: Claude Code ends a Stop hook at 10 s";
    const MINUTE: Duration = Duration::from_secs(60);
    const HOUR_OF_POLLS: usize = 61;

    #[test]
    fn the_poll_interval_and_the_overdue_mark_are_the_configured_millis() {
        let settings = CiSettings::try_from(&toml::from_str::<Config>(CONFIGURED).unwrap()).unwrap();
        let overdue = [8, 9].map(|millis| settings.run_overdue(Duration::from_millis(millis)));
        assert_eq!((settings.poll_interval().duration(), overdue), (Duration::from_millis(7), [None, Some(())]));
    }

    #[test]
    fn a_stop_limit_at_seven_seconds_or_more_is_refused() {
        let configs = STOP_LIMITS.map(|text| toml::from_str::<Config>(text).unwrap());
        let [under, at] = configs.each_ref().map(CiSettings::try_from);
        assert!(under.is_ok_and(|settings| settings.stop_limit().deadline() > std::time::Instant::now()));
        assert!(at.unwrap_err().to_string().contains(REFUSED));
    }

    #[test]
    fn the_queue_pace_meets_its_floor_and_bounds_the_polls_for_one_head() {
        let settings = CiSettings::try_from(&Config::default()).unwrap();
        let paced = [MINUTE, MINUTE + Duration::from_millis(1)].map(|floor| settings.queue_pace(floor).ok());
        assert_eq!(paced, [Some(MINUTE), None]);
        assert_eq!(settings.queue_turns(MINUTE).count(), HOUR_OF_POLLS);
    }
}
