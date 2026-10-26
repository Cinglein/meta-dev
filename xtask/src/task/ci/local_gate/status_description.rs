use std::fmt;

use crate::task::ci::local_gate::GateState;

#[derive(Debug, PartialEq, Eq)]
pub struct StatusDescription {
    version: String,
    toolchain: String,
    state: GateState,
    times: String,
    surfaced: String,
    names: Vec<String>,
}

impl StatusDescription {
    pub const MAX_LENGTH: usize = 140;
    const PASSED: &str = "; env passed: ";
    const FAILED: &str = "; failed ";
    const FAILED_SUFFIX: &str = " failed";
    const SEPARATOR: &str = ", ";
    const INCOMPLETE: &str = "incomplete";
    const ELLIPSIS: &str = "...";
    const MIN_TOOLCHAIN_LEN: usize = 12;

    pub fn describing(version: &str, toolchain: &str) -> StatusDescription {
        StatusDescription {
            version: version.to_owned(),
            toolchain: toolchain.to_owned(),
            state: GateState::Success,
            times: String::new(),
            surfaced: String::new(),
            names: Vec::new(),
        }
    }

    #[must_use]
    pub fn after(self, state: GateState, times: &str) -> StatusDescription {
        StatusDescription { state, times: times.to_owned(), ..self }
    }

    #[must_use]
    pub fn surfacing(self, surfaced: &str) -> StatusDescription {
        StatusDescription { surfaced: surfaced.to_owned(), ..self }
    }

    #[must_use]
    pub fn passing(self, names: &[&str]) -> StatusDescription {
        StatusDescription { names: names.iter().map(|name| (*name).to_owned()).collect(), ..self }
    }

    fn fitted(&self) -> String {
        let lead = format!("meta-dev {} on ", self.version);
        let marks = format!("{}{}", self.failure().unwrap_or_default(), self.surfaced);
        let names: Vec<&str> = self.names.iter().map(String::as_str).collect();
        let whole = format!("{lead}{}: {}{marks}{}", self.toolchain, self.times, Self::listed(&names, &[]));
        std::iter::once(whole)
            .filter(|text| text.chars().count() <= Self::MAX_LENGTH)
            .chain((0..=names.len()).rev().filter_map(|kept_len| {
                let (shown, hidden) = names.split_at(kept_len);
                self.shortened(&lead, &marks, shown, hidden)
            }))
            .next()
            .unwrap_or_else(|| format!("{lead}{}{marks}{}", Self::ELLIPSIS, Self::listed(&[], &names)))
    }

    fn shortened(&self, lead: &str, marks: &str, shown: &[&str], hidden: &[&str]) -> Option<String> {
        let tail = format!("{marks}{}", Self::listed(shown, hidden));
        let room = Self::MAX_LENGTH.saturating_sub(lead.chars().count() + tail.chars().count());
        let toolchain = match self.toolchain.chars().count() {
            full if full <= room => Some(self.toolchain.clone()),
            _ if room >= Self::MIN_TOOLCHAIN_LEN => {
                let kept: String = self.toolchain.chars().take(room - Self::ELLIPSIS.len()).collect();
                Some(format!("{kept}{}", Self::ELLIPSIS))
            }
            _ => None,
        };
        toolchain.map(|toolchain| format!("{lead}{toolchain}{tail}"))
    }

    fn failure(&self) -> Option<String> {
        let step = self
            .times
            .split(Self::SEPARATOR)
            .find_map(|entry| entry.strip_suffix(Self::FAILED_SUFFIX))
            .unwrap_or(Self::INCOMPLETE);
        match self.state {
            GateState::Success => None,
            GateState::Failure => Some(format!("{}{step}", Self::FAILED)),
        }
    }

    fn listed(shown: &[&str], hidden: &[&str]) -> String {
        let more = (!hidden.is_empty()).then(|| format!("+{}", hidden.len()));
        let items: Vec<&str> = shown.iter().copied().chain(more.as_deref()).collect();
        if items.is_empty() { String::new() } else { format!("{}{}", Self::PASSED, items.join(Self::SEPARATOR)) }
    }
}

impl fmt::Display for StatusDescription {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.fitted())
    }
}

#[cfg(test)]
mod tests {
    use super::StatusDescription;
    use crate::task::ci::local_gate::GateState;

    const VERSION: &str = "be728d0";
    const PREFIX: &str = "meta-dev be728d0 on ";
    const APPLE: &str = "stable-aarch64-apple-darwin";
    const ALL_TIMES: &str = "fmt 160.5ms, clippy 1.8s, test 317.9ms, test-time 55.2ms, lint 47.2ms, spec 41.0ns";
    const FAILED_TIMES: &str = "fmt 160.5ms, clippy 1.8s, test failed";
    const PARTIAL_TIMES: &str = "fmt 160.5ms, clippy 1.8s";
    const NOTHING: &str = "";
    const SURFACED: &str = "; surfaces changed";
    const FAILURE: &str = "; failed ";
    const LONG: &str = "stable-aarch64-apple-darwinstable-aarch64-apple-darwinstable-aarch64-apple-darwinstable-aarch64-apple-darwinstable-aarch64-apple-darwin";
    const EXPECT_TIMED: &str = "meta-dev be728d0 on stable-aarch64-apple-darwin: fmt 160.5ms, clippy 1.8s, test failed; failed test; surfaces changed; env passed: CHROME";
    const EXPECT_DROPPED: &str = "meta-dev be728d0 on stable-aarch64-apple-darwin; surfaces changed";
    const EXPECT_EMPTY: &str = "meta-dev be728d0 on stable-aarch64-apple-darwin: ; failed incomplete; surfaces changed";
    const EXPECT_PAIR: &str = "meta-dev be728d0 on stable-aarch64-apple-darwinstable-aarch64-apple-darwin...; failed incomplete; surfaces changed; env passed: CHROME, TERM";
    const EXPECT_WIDE: &str = "meta-dev be728d0 on stable-aarch64-apple-darwinstable-...; failed incomplete; surfaces changed; env passed: A234567890123456789012345678, +2";
    const NAMES: [&str; 4] = ["CHROME", "TERM", "DISPLAY_NAME", "SSH_AUTH_SOCK"];
    const WIDE: [&str; 3] =
        ["A234567890123456789012345678", "B234567890123456789012345678", "C234567890123456789012345678"];

    #[test]
    fn every_description_fits_and_keeps_the_rev_the_failure_and_the_surfaces() {
        let describe = |toolchain: &str, state, times: &str, names: &[&str]| {
            StatusDescription::describing(VERSION, toolchain)
                .after(state, times)
                .surfacing(SURFACED)
                .passing(names)
                .to_string()
        };
        let cases: Vec<(GateState, String)> = [APPLE, LONG]
            .into_iter()
            .flat_map(|toolchain| [ALL_TIMES, FAILED_TIMES, PARTIAL_TIMES, NOTHING].map(|times| (toolchain, times)))
            .flat_map(|(toolchain, times)| {
                [GateState::Success, GateState::Failure].map(|state| (toolchain, times, state))
            })
            .flat_map(|(toolchain, times, state)| {
                (0..=NAMES.len()).map(move |count| (state, describe(toolchain, state, times, &NAMES[..count])))
            })
            .collect();
        assert!(cases.iter().all(|(_, shown)| shown.chars().count() <= StatusDescription::MAX_LENGTH));
        assert!(cases.iter().all(|(_, shown)| shown.starts_with(PREFIX) && shown.contains(SURFACED)));
        assert!(cases.iter().all(|(state, shown)| (*state == GateState::Failure) == shown.contains(FAILURE)));
    }

    #[test]
    fn times_go_first_and_a_failure_names_its_step_or_says_incomplete() {
        let stated = |state, times: &str, names: &[&str]| {
            StatusDescription::describing(VERSION, APPLE)
                .after(state, times)
                .surfacing(SURFACED)
                .passing(names)
                .to_string()
        };
        assert_eq!(stated(GateState::Failure, FAILED_TIMES, &NAMES[..1]), EXPECT_TIMED);
        assert_eq!(stated(GateState::Success, ALL_TIMES, &[]), EXPECT_DROPPED);
        assert_eq!(stated(GateState::Failure, NOTHING, &[]), EXPECT_EMPTY);
    }

    #[test]
    fn the_toolchain_shortens_first_then_the_env_list_and_the_markers_stay() {
        let describe = |names: &[&str]| {
            StatusDescription::describing(VERSION, LONG)
                .after(GateState::Failure, PARTIAL_TIMES)
                .surfacing(SURFACED)
                .passing(names)
                .to_string()
        };
        assert_eq!(describe(&NAMES[..2]), EXPECT_PAIR);
        assert_eq!(describe(&WIDE), EXPECT_WIDE);
    }
}
