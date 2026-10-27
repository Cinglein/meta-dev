use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use task::{BranchName, CommandLine, GateEnvName, Program, Site};
use tempfile::TempDir;

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
    branch: Option<BranchName>,
}

impl GateChild {
    const INHERITED: [&str; 3] = ["PATH", "TMPDIR", "SCCACHE_DIR"];
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
    pub fn heading(self, branch: Option<BranchName>) -> GateChild {
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
    use std::collections::BTreeMap;
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;

    use task::{CommandLine, GateEnvName, Program, Stdout};

    use super::GateChild;
    use crate::task::ci::run_kind::RunKind;
    use crate::task::fixture::Fixture;

    const SHELL: &str = "sh";
    const SHOWN: [&str; 2] = ["-c", "printf %s \"$HOME|$XDG_CONFIG_HOME|$RUSTUP_TOOLCHAIN|$CARGO_TARGET_DIR\""];
    const PRINT: [&str; 2] = ["-c", "printf %s \"$(printenv \"$0\")\""];
    const TOOLCHAIN: &str = "1.90-x86_64-unknown-linux-gnu";
    const TARGET: &str = "gate-target";
    const KNOWN: &str = "GATE_KNOWN_NAME";
    const KNOWN_VALUE: &str = "known7value";
    const BRANCH: &str = "task-00000000000000a1";

    #[test]
    fn the_child_gets_an_empty_home_of_its_own_no_config_home_and_only_allowed_names_with_the_callers_values() {
        let fixture = Fixture::new();
        fs::create_dir_all(fixture.beside()).unwrap();
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
        let fresh = fs::read_dir(home).is_ok_and(|mut entries| entries.next().is_none());
        assert!(fresh && env::home_dir().is_none_or(|real| real != home) && home.starts_with(fixture.beside()));
        let printed = [(&passing, KNOWN), (&plain, KNOWN), (&passing, RunKind::GATE_BRANCH)]
            .map(|(child, variable)| shown(child, PRINT, variable));
        assert_eq!(printed, [KNOWN_VALUE, "", BRANCH]);
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
