use std::fmt;
use std::path::Path;

use task::{GateEnvName, GateStatus};

use crate::task::ci::local_gate::{GateState, MetaRev, RecordedTimes, Toolchain};
use crate::task::revision::Commit;
use crate::task::surfaces::Changed;

#[derive(Debug)]
pub enum StatusArgument<'post> {
    Endpoint(&'post Commit),
    State(GateState),
    Context {
        status: &'post GateStatus,
        scope: &'post Path,
    },
    Description {
        version: MetaRev,
        toolchain: &'post Toolchain,
        times: &'post RecordedTimes,
        surfaces: &'post Changed,
        passed: &'post [GateEnvName],
    },
}

impl StatusArgument<'_> {
    const SURFACES: &str = "; surfaces changed";
    const PASSED: &str = "; env passed: ";
}

impl fmt::Display for StatusArgument<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusArgument::Endpoint(commit) => write!(formatter, "repos/{{owner}}/{{repo}}/statuses/{commit}"),
            StatusArgument::State(state) => write!(formatter, "state={state}"),
            StatusArgument::Context { status, scope } if scope.as_os_str().is_empty() => {
                write!(formatter, "context={status}")
            }
            StatusArgument::Context { status, scope } => write!(formatter, "context={status}/{}", scope.display()),
            StatusArgument::Description { version, toolchain, times, surfaces, passed } => {
                let head = format!("meta-dev {version} on {toolchain}");
                let names: Vec<&str> = passed.iter().map(GateEnvName::as_str).collect();
                let surfaced = if surfaces.is_empty() { "" } else { Self::SURFACES };
                write!(formatter, "description={}", Self::fitted(&head, &times.to_string(), surfaced, &names))
            }
        }
    }
}

impl StatusArgument<'_> {
    const LIMIT: usize = 140;
    const FAILED: &str = "; failed ";
    const FAILED_SUFFIX: &str = " failed";
    const SEPARATOR: &str = ", ";

    fn listed(names: &[&str], hidden: usize) -> String {
        let more = (hidden > 0).then(|| format!("+{hidden}"));
        let items: Vec<&str> = names.iter().copied().chain(more.as_deref()).collect();
        if items.is_empty() { String::new() } else { format!("{}{}", Self::PASSED, items.join(Self::SEPARATOR)) }
    }

    fn fitted(head: &str, times: &str, surfaced: &str, names: &[&str]) -> String {
        let failed = times
            .split(Self::SEPARATOR)
            .find_map(|entry| entry.strip_suffix(Self::FAILED_SUFFIX).and_then(|_| entry.split(' ').next()))
            .map(|step| format!("{}{step}", Self::FAILED))
            .unwrap_or_default();
        let whole = format!("{head}: {times}{surfaced}{}", Self::listed(names, 0));
        let without = |kept: usize| {
            let (shown, hidden) = names.split_at(kept);
            format!("{head}{failed}{surfaced}{}", Self::listed(shown, hidden.len()))
        };
        std::iter::once(whole)
            .chain((0..=names.len()).rev().map(without))
            .find(|text| text.chars().count() <= Self::LIMIT)
            .unwrap_or_else(|| without(0).chars().take(Self::LIMIT).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use settings::EmbeddedDefaults;
    use task::{GateEnvName, Policy};

    use super::StatusArgument;
    use crate::task::ci::local_gate::{MetaRev, RecordedTimes, Toolchain};
    use crate::task::surfaces::Changed;

    const VERSION: &str = "0a1b2c";
    const ANY: &str = "Cargo.toml";
    const NAMES: [&str; 2] = ["CHROME", "TERM"];
    const ROOT_CONTEXT: &str = "context=local-gate";
    const UNNOTED: &str = "description=meta-dev 0a1b2c on 1.90-x86_64-unknown-linux-gnu: ";
    const TOOLCHAIN: &str = "1.90-x86_64-unknown-linux-gnu (default)";

    #[test]
    fn the_root_workspace_keeps_the_bare_status_and_unchanged_surfaces_add_no_note() {
        let policy = Policy::defaults();
        let context = StatusArgument::Context { status: policy.gate().unwrap(), scope: Path::new("") };
        let (times, unchanged) = (RecordedTimes::new(PathBuf::new()), Changed::new(Vec::new()));
        let version: MetaRev = VERSION.parse().unwrap();
        let toolchain: Toolchain = TOOLCHAIN.parse().unwrap();
        let description = StatusArgument::Description {
            version,
            toolchain: &toolchain,
            times: &times,
            surfaces: &unchanged,
            passed: &[],
        };
        assert_eq!(context.to_string(), ROOT_CONTEXT);
        assert_eq!(description.to_string(), UNNOTED);
    }

    const LIMIT: usize = 140;
    const ALL_TIMES: &str = "fmt 160.5ms, clippy 1.8s, test 317.9ms, test-time 55.2ms, lint 47.2ms, spec 41.0ns";
    const FAILED_TIMES: &str = "fmt 160.5ms, clippy 1.8s, test failed";
    const APPLE: &str = "stable-aarch64-apple-darwin";
    const MANY: [&str; 4] = ["CHROME", "TERM", "DISPLAY_NAME", "SSH_AUTH_SOCK"];

    fn described(times: &str, toolchain: &str, surfaces: bool, names: &[&str]) -> String {
        let directory = tempfile::TempDir::new().unwrap();
        let path = directory.path().join("times");
        std::fs::write(&path, times).unwrap();
        let recorded = RecordedTimes::new(path);
        let changed = Changed::new(if surfaces { vec![PathBuf::from(ANY)] } else { Vec::new() });
        let passed: Vec<GateEnvName> = names.iter().map(|name| name.to_string().try_into().unwrap()).collect();
        StatusArgument::Description {
            version: "be728d0".parse().unwrap(),
            toolchain: &toolchain.parse().unwrap(),
            times: &recorded,
            surfaces: &changed,
            passed: &passed,
        }
        .to_string()
        .strip_prefix("description=")
        .unwrap()
        .to_owned()
    }

    #[test]
    fn every_description_fits_and_keeps_the_state_bearing_parts() {
        let mut checked = 0;
        for times in [ALL_TIMES, FAILED_TIMES, ""] {
            for surfaces in [false, true] {
                for count in 0..=MANY.len() {
                    let shown = described(times, APPLE, surfaces, &MANY[..count]);
                    assert!(shown.chars().count() <= LIMIT, "{shown}");
                    assert!(shown.starts_with("meta-dev be728d0 on "));
                    assert!(shown.contains(APPLE) && (surfaces == shown.contains("surfaces changed")));
                    assert!(
                        !times.contains("failed") || shown.contains("test failed") || shown.contains("failed test")
                    );
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 30);
    }

    #[test]
    fn per_step_times_go_first_when_the_description_is_too_long() {
        let long = described(ALL_TIMES, APPLE, true, &[]);
        assert_eq!(long, format!("meta-dev be728d0 on {APPLE}; surfaces changed"));
        let failed = described(FAILED_TIMES, APPLE, true, &["CHROME"]);
        assert_eq!(
            failed,
            format!("meta-dev be728d0 on {APPLE}: {FAILED_TIMES}; surfaces changed; env passed: CHROME")
        );
        let tight = described(&format!("{FAILED_TIMES}, lint 1.0s, spec 1.0s"), APPLE, true, &MANY);
        assert_eq!(
            tight,
            format!(
                "meta-dev be728d0 on {APPLE}; failed test; surfaces changed; env passed: CHROME, TERM, DISPLAY_NAME, SSH_AUTH_SOCK"
            )
        );
        let dropped = described(ALL_TIMES, APPLE, true, &MANY);
        assert_eq!(
            dropped,
            format!(
                "meta-dev be728d0 on {APPLE}; surfaces changed; env passed: CHROME, TERM, DISPLAY_NAME, SSH_AUTH_SOCK"
            )
        );
    }

    #[test]
    fn names_that_cannot_fit_are_counted_and_a_huge_toolchain_is_cut_at_the_limit() {
        let names = ["AAAAAAAAAAAAAAAAAAAAAAAAAAAA", "BBBBBBBBBBBBBBBBBBBBBBBBBBBB", "CCCCCCCCCCCCCCCCCCCCCCCCCCCC"];
        let counted = described(ALL_TIMES, APPLE, true, &names);
        assert_eq!(counted, format!("meta-dev be728d0 on {APPLE}; surfaces changed; env passed: {}, +2", names[0]));
        let cut = described(ALL_TIMES, &"x".repeat(300), true, &names);
        assert_eq!(cut.chars().count(), LIMIT);
    }

    #[test]
    fn passed_names_are_listed_after_the_surfaces_note() {
        let (times, changed) = (RecordedTimes::new(PathBuf::new()), Changed::new(vec![PathBuf::from(ANY)]));
        let passed: Vec<GateEnvName> = NAMES.iter().map(|name| name.to_string().try_into().unwrap()).collect();
        let description = StatusArgument::Description {
            version: VERSION.parse().unwrap(),
            toolchain: &TOOLCHAIN.parse().unwrap(),
            times: &times,
            surfaces: &changed,
            passed: &passed,
        };
        assert_eq!(description.to_string(), format!("{UNNOTED}; surfaces changed; env passed: CHROME, TERM"));
    }
}
