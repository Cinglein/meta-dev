use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct StatusDescription(String);

impl StatusDescription {
    pub const MAX_LENGTH: usize = 140;
    const PASSED: &str = "; env passed: ";
    const FAILED: &str = "; failed ";
    const FAILED_SUFFIX: &str = " failed";
    const SEPARATOR: &str = ", ";
    const STEP_END: char = ' ';

    pub fn new(head: &str, times: &str, surfaced: &str, names: &[&str]) -> StatusDescription {
        let failed = times
            .split(Self::SEPARATOR)
            .find_map(|entry| entry.strip_suffix(Self::FAILED_SUFFIX).and_then(|_| entry.split(Self::STEP_END).next()))
            .map(|step| format!("{}{step}", Self::FAILED))
            .unwrap_or_default();
        let whole = format!("{head}: {times}{surfaced}{}", Self::listed(names, &[]));
        let without = |kept_len: usize| {
            let (shown, hidden) = names.split_at(kept_len);
            format!("{head}{failed}{surfaced}{}", Self::listed(shown, hidden))
        };
        StatusDescription(
            std::iter::once(whole)
                .chain((0..=names.len()).rev().map(without))
                .find(|text| text.chars().count() <= Self::MAX_LENGTH)
                .unwrap_or_else(|| without(0).chars().take(Self::MAX_LENGTH).collect()),
        )
    }

    fn listed(shown: &[&str], hidden: &[&str]) -> String {
        let more = (!hidden.is_empty()).then(|| format!("+{}", hidden.len()));
        let items: Vec<&str> = shown.iter().copied().chain(more.as_deref()).collect();
        if items.is_empty() { String::new() } else { format!("{}{}", Self::PASSED, items.join(Self::SEPARATOR)) }
    }
}

impl fmt::Display for StatusDescription {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::StatusDescription;

    const HEAD: &str = "meta-dev be728d0 on stable-aarch64-apple-darwin";
    const ALL_TIMES: &str = "fmt 160.5ms, clippy 1.8s, test 317.9ms, test-time 55.2ms, lint 47.2ms, spec 41.0ns";
    const FAILED_TIMES: &str = "fmt 160.5ms, clippy 1.8s, test failed";
    const LONG_FAILED_TIMES: &str = "fmt 160.5ms, clippy 1.8s, test failed, lint 1.0s, spec 1.0s";
    const SURFACED: &str = "; surfaces changed";
    const NOTHING: &str = "";
    const JOIN: &str = ", ";
    const FAILED: &str = "failed";
    const STEP: &str = "test";
    const NAMES: [&str; 4] = ["CHROME", "TERM", "DISPLAY_NAME", "SSH_AUTH_SOCK"];
    const WIDE: [&str; 3] =
        ["A234567890123456789012345678", "B234567890123456789012345678", "C234567890123456789012345678"];
    const LISTED: &str = "; env passed: CHROME, TERM, DISPLAY_NAME, SSH_AUTH_SOCK";

    #[test]
    fn every_description_fits_and_keeps_the_state_bearing_parts() {
        let cases = [ALL_TIMES, FAILED_TIMES, LONG_FAILED_TIMES, ""]
            .into_iter()
            .flat_map(|times| [(NOTHING, times), (SURFACED, times)])
            .flat_map(|(surfaced, times)| (0..=NAMES.len()).map(move |count| (surfaced, times, count)))
            .map(|(surfaced, times, count)| {
                (surfaced, times, count, StatusDescription::new(HEAD, times, surfaced, &NAMES[..count]).to_string())
            })
            .collect::<Vec<_>>();
        assert!(cases.iter().all(|(.., shown)| shown.chars().count() <= StatusDescription::MAX_LENGTH));
        assert!(cases.iter().all(|(surfaced, _, _, shown)| shown.starts_with(HEAD) && shown.contains(surfaced)));
        assert!(cases.iter().all(|(_, times, count, shown)| {
            shown.contains(&NAMES[..*count].join(JOIN)) && (!times.contains(FAILED) || shown.contains(STEP))
        }));
    }

    #[test]
    fn per_step_times_go_first_and_a_failed_step_stays() {
        let describe = |times: &str, names: &[&str]| StatusDescription::new(HEAD, times, SURFACED, names).to_string();
        assert_eq!(describe(ALL_TIMES, &[]), format!("{HEAD}{SURFACED}"));
        assert_eq!(
            describe(FAILED_TIMES, &NAMES[..1]),
            format!("{HEAD}: {FAILED_TIMES}{SURFACED}; env passed: CHROME")
        );
        assert_eq!(describe(LONG_FAILED_TIMES, &NAMES), format!("{HEAD}; failed test{SURFACED}{LISTED}"));
    }

    #[test]
    fn names_that_cannot_fit_are_counted_and_the_rest_is_cut_at_the_limit() {
        let describe = |head: &str| StatusDescription::new(head, ALL_TIMES, SURFACED, &WIDE).to_string();
        assert_eq!(describe(HEAD), format!("{HEAD}{SURFACED}; env passed: {}, +2", WIDE[0]));
        let huge = HEAD.repeat(StatusDescription::MAX_LENGTH);
        assert_eq!(describe(&huge).chars().count(), StatusDescription::MAX_LENGTH);
    }
}
