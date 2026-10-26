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
    const STEP_END: char = ' ';

    pub fn of(version: &str, toolchain: &str) -> StatusDescription {
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
        let head = format!("meta-dev {} on {}", self.version, self.toolchain);
        let (times, surfaced) = (&self.times, &self.surfaced);
        let names: Vec<&str> = self.names.iter().map(String::as_str).collect();
        let failed = times
            .split(Self::SEPARATOR)
            .find_map(|entry| entry.strip_suffix(Self::FAILED_SUFFIX).and_then(|_| entry.split(Self::STEP_END).next()))
            .map(|step| format!("{}{step}", Self::FAILED))
            .unwrap_or_default();
        let whole = format!("{head}: {times}{surfaced}{}", Self::listed(&names, &[]));
        let without = |kept_len: usize| {
            let (shown, hidden) = names.split_at(kept_len);
            format!("{head}{failed}{surfaced}{}", Self::listed(shown, hidden))
        };
        std::iter::once(whole)
            .chain((0..=names.len()).rev().map(without))
            .find(|text| text.chars().count() <= Self::MAX_LENGTH)
            .unwrap_or_else(|| without(0).chars().take(Self::MAX_LENGTH).collect())
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
    const INCOMPLETE: &str = "; failed incomplete";
    const ELLIPSIS: &str = "...";
    const NAMES: [&str; 4] = ["CHROME", "TERM", "DISPLAY_NAME", "SSH_AUTH_SOCK"];
    const WIDE: [&str; 3] =
        ["A234567890123456789012345678", "B234567890123456789012345678", "C234567890123456789012345678"];
    const PAIR: &str = "; env passed: CHROME, TERM";

    #[test]
    fn every_description_fits_and_keeps_the_rev_the_failure_and_the_surfaces() {
        let describe = |toolchain: &str, state, times: &str, names: &[&str]| {
            StatusDescription::of(VERSION, toolchain).after(state, times).surfacing(SURFACED).passing(names).to_string()
        };
        let long = APPLE.repeat(NAMES.len() + 1);
        let cases: Vec<(GateState, String)> = [APPLE, long.as_str()]
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
            StatusDescription::of(VERSION, APPLE).after(state, times).surfacing(SURFACED).passing(names).to_string()
        };
        let timed = format!("{PREFIX}{APPLE}: {FAILED_TIMES}{FAILURE}test{SURFACED}; env passed: CHROME");
        assert_eq!(stated(GateState::Failure, FAILED_TIMES, &NAMES[..1]), timed);
        assert_eq!(stated(GateState::Success, ALL_TIMES, &[]), format!("{PREFIX}{APPLE}{SURFACED}"));
        assert_eq!(stated(GateState::Failure, NOTHING, &[]), format!("{PREFIX}{APPLE}: {INCOMPLETE}{SURFACED}"));
    }

    #[test]
    fn the_toolchain_shortens_first_then_the_env_list_and_the_markers_stay() {
        let long = APPLE.repeat(NAMES.len() + 1);
        let describe = |names: &[&str]| {
            StatusDescription::of(VERSION, &long)
                .after(GateState::Failure, PARTIAL_TIMES)
                .surfacing(SURFACED)
                .passing(names)
                .to_string()
        };
        let pair = describe(&NAMES[..2]);
        assert_eq!(pair, format!("{PREFIX}{}{ELLIPSIS}{INCOMPLETE}{SURFACED}{PAIR}", &long[..54]));
        let wide = describe(&WIDE);
        let tail = format!("{INCOMPLETE}{SURFACED}; env passed: {}, +2", WIDE[0]);
        assert_eq!(wide, format!("{PREFIX}{}{ELLIPSIS}{tail}", &long[..34]));
    }
}
