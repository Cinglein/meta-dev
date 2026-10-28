use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use task::{CommandLine, GateEnvName, Program, Site};
use tempfile::TempDir;

use crate::task::ci::head_branch::HeadBranch;
use crate::task::ci::local_gate::{LocalGate, Toolchain};
use crate::task::ci::run_kind::RunKind;
use crate::task::failure::Failure;

#[derive(Debug)]
pub struct GateChild {
    home: TempDir,
    cargo_home: PathBuf,
    rustup_home: PathBuf,
    target: PathBuf,
    times: PathBuf,
    toolchain: Toolchain,
    passed: Vec<GateEnvName>,
    ambient: BTreeMap<String, OsString>,
    branch: Option<HeadBranch>,
}

impl GateChild {
    const INHERITED: [&str; 4] = ["PATH", "TMPDIR", "SCCACHE_DIR", RunKind::NUMBER];
    const HOME: &str = "HOME";
    const CARGO_HOME: &str = "CARGO_HOME";
    const RUSTUP_HOME: &str = "RUSTUP_HOME";
    const DOT_CARGO: &str = ".cargo";
    const DOT_RUSTUP: &str = ".rustup";
    const TOOLCHAIN: &str = "RUSTUP_TOOLCHAIN";
    const TARGET: &str = "CARGO_TARGET_DIR";
    const BUILD_DIR: &str = "CARGO_BUILD_BUILD_DIR";

    pub fn new(parent: &Path, target: PathBuf, times: PathBuf, toolchain: Toolchain) -> Result<GateChild, Failure> {
        let home = TempDir::new_in(parent)
            .map_err(|error| task::Failure::Io { site: Site::File(parent.to_path_buf()), error })?;
        let real_home = env::home_dir().unwrap_or_default();
        let real =
            |name: &str, fallback: &str| env::var_os(name).map_or_else(|| real_home.join(fallback), PathBuf::from);
        Ok(GateChild {
            home,
            cargo_home: real(Self::CARGO_HOME, Self::DOT_CARGO),
            rustup_home: real(Self::RUSTUP_HOME, Self::DOT_RUSTUP),
            target,
            times,
            toolchain,
            passed: Vec::new(),
            ambient: BTreeMap::new(),
            branch: None,
        })
    }

    #[must_use]
    pub fn heading(self, branch: Option<HeadBranch>) -> GateChild {
        GateChild { branch, ..self }
    }

    #[must_use]
    pub fn seeing(self, ambient: BTreeMap<String, OsString>) -> GateChild {
        GateChild { ambient, ..self }
    }

    #[must_use]
    pub fn passing(self, passed: &[GateEnvName]) -> GateChild {
        GateChild { passed: passed.to_vec(), ..self }
    }

    pub fn cargo_home(&self) -> &Path {
        &self.cargo_home
    }

    pub fn line(&self, program: Program, directory: PathBuf) -> CommandLine {
        CommandLine::new(program)
            .in_directory(directory)
            .inheriting_only_from(
                &self.ambient,
                Self::INHERITED.into_iter().chain(self.passed.iter().map(GateEnvName::as_str)),
            )
            .environment(Self::HOME, self.home.path())
            .environment(Self::CARGO_HOME, &self.cargo_home)
            .environment(Self::RUSTUP_HOME, &self.rustup_home)
            .environment(Self::TOOLCHAIN, &self.toolchain)
            .environment(Self::TARGET, &self.target)
            .environment(Self::BUILD_DIR, &self.target)
            .environment(LocalGate::TIMES, &self.times)
            .environment(RunKind::GATE_BRANCH, self.branch.as_ref().map(ToString::to_string).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use meta_door::{Change, Door};
    use std::collections::BTreeMap;
    use std::env;
    use std::ffi::OsString;
    use std::path::PathBuf;

    use task::{CommandLine, GateEnvName, Program, Stdout};

    use super::GateChild;
    use crate::task::ci::run_kind::RunKind;
    use crate::task::ci::tracked_work::TrackedWork;
    use crate::task::fixture::Fixture;

    const SHELL: &str = "sh";
    const SHOWN: [&str; 2] = ["-c", "printf %s \"$HOME|$XDG_CONFIG_HOME|$RUSTUP_TOOLCHAIN|$CARGO_TARGET_DIR\""];
    const PRINT: [&str; 2] = ["-c", "printf %s \"$(printenv \"$0\" >/dev/null && printf +)$(printenv \"$0\")\""];
    const TOOLCHAIN: &str = "1.90-x86_64-unknown-linux-gnu";
    const TARGET: &str = "gate-target";
    const KNOWN: &str = "GATE_KNOWN_NAME";
    const KNOWN_VALUE: &str = "known7value";
    const BRANCH: &str = "task-00000000000000a1";
    const PRINTED: [&str; 4] = ["+known7value", "", "+task-00000000000000a1", "+"];
    const NUMBER: (&str, &str) = ("PULL_REQUEST", "7");
    const NUMBER_SHOWN: [&str; 2] = ["-c", "printf %s \"$PULL_REQUEST\""];
    const DONE: &str = "task-00000000000000d4";

    #[test]
    fn the_child_gets_an_empty_home_of_its_own_no_config_home_and_only_allowed_names_with_the_callers_values() {
        let fixture = Fixture::new();
        Door::changed(fixture.beside(), Change::DirAll).unwrap();
        let name = GateEnvName::try_from(KNOWN.to_owned()).unwrap();
        let ambient: BTreeMap<String, OsString> =
            CommandLine::ambient().into_iter().chain([(KNOWN.to_owned(), OsString::from(KNOWN_VALUE))]).collect();
        let child = |toolchain| GateChild::new(&fixture.beside(), TARGET.into(), PathBuf::new(), toolchain).unwrap();
        let shown = |child: &GateChild, script: [&str; 2], variable: &str| {
            let line = child.line(Program::Executable(SHELL.into()), fixture.beside());
            line.arguments(script).argument(variable).captured::<Stdout>().unwrap_or_default()
        };
        let plain = child(TOOLCHAIN.parse().unwrap()).seeing(ambient.clone());
        let passing = child(TOOLCHAIN.parse().unwrap()).seeing(ambient).passing(std::slice::from_ref(&name));
        let (home, passing) = (plain.home.path(), passing.heading(BRANCH.parse().ok()));
        assert_eq!(shown(&plain, SHOWN, KNOWN), format!("{}||{TOOLCHAIN}|{TARGET}", home.display()));
        let fresh = Door::listed(home).is_ok_and(|entries| entries.is_empty());
        assert!(fresh && env::home_dir().is_none_or(|real| real != home) && home.starts_with(fixture.beside()));
        let printed =
            [(&passing, KNOWN), (&plain, KNOWN), (&passing, RunKind::GATE_BRANCH), (&plain, RunKind::GATE_BRANCH)]
                .map(|(child, variable)| shown(child, PRINT, variable));
        assert_eq!(printed, PRINTED);
    }

    #[test]
    fn after_meta_done_a_child_given_the_open_pull_request_passes_the_linked_task() {
        let fixture = Fixture::new();
        Door::changed(fixture.beside(), Change::DirAll).unwrap();
        let ambient = BTreeMap::from([(NUMBER.0.to_owned(), OsString::from(NUMBER.1))]);
        let child = GateChild::new(&fixture.beside(), TARGET.into(), PathBuf::new(), TOOLCHAIN.parse().unwrap());
        let line = child.unwrap().seeing(ambient).line(Program::Executable(SHELL.into()), fixture.beside());
        let seen = line.arguments(NUMBER_SHOWN).captured::<Stdout>().unwrap_or_default();
        assert!(TrackedWork::judged(Some(DONE), (NUMBER.0, &seen)).is_ok());
    }

    #[test]
    fn the_ambient_environment_is_this_process_environment() {
        let ambient = CommandLine::ambient();
        let process: Vec<(String, OsString)> =
            env::vars_os().filter_map(|(name, value)| name.into_string().ok().map(|name| (name, value))).collect();
        assert_eq!(ambient.len(), process.len());
        assert!(process.iter().all(|(name, value)| ambient.get(name) == Some(value)));
    }
}
