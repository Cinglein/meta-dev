use std::env;
use std::path::{Path, PathBuf};

use task::{CommandLine, Program, Site};
use tempfile::TempDir;

use crate::task::ci::local_gate::{LocalGate, Toolchain};
use crate::task::failure::Failure;

#[derive(Debug)]
pub struct GateChild {
    home: TempDir,
    cargo_home: PathBuf,
    rustup_home: PathBuf,
    target: PathBuf,
    times: PathBuf,
    toolchain: Toolchain,
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
        })
    }

    pub fn cargo_home(&self) -> &Path {
        &self.cargo_home
    }

    pub fn line(&self, program: Program, directory: PathBuf) -> CommandLine {
        CommandLine::new(program)
            .in_directory(directory)
            .inheriting_only(&Self::INHERITED)
            .environment(Self::HOME, self.home.path().parent().unwrap_or(self.home.path()))
            .environment(Self::CARGO_HOME, &self.cargo_home)
            .environment(Self::RUSTUP_HOME, &self.rustup_home)
            .environment(Self::TOOLCHAIN, &self.toolchain)
            .environment(Self::TARGET, &self.target)
            .environment(Self::BUILD_DIR, &self.target)
            .environment(LocalGate::TIMES, &self.times)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    use task::{Program, Stdout};

    use super::GateChild;
    use crate::task::fixture::Fixture;

    const SHELL: &str = "sh";
    const SHOWN: [&str; 2] = ["-c", "printf %s \"$HOME|$XDG_CONFIG_HOME|$RUSTUP_TOOLCHAIN|$CARGO_TARGET_DIR\""];
    const TOOLCHAIN: &str = "1.90-x86_64-unknown-linux-gnu";
    const TARGET: &str = "gate-target";

    #[test]
    fn the_child_gets_an_empty_home_of_its_own_and_no_config_home() {
        let fixture = Fixture::new();
        fs::create_dir_all(fixture.beside()).unwrap();
        let toolchain = TOOLCHAIN.parse().unwrap();
        let child = GateChild::new(&fixture.beside(), PathBuf::from(TARGET), PathBuf::new(), toolchain).unwrap();
        let shown =
            child.line(Program::Executable(SHELL.into()), fixture.beside()).arguments(SHOWN).captured::<Stdout>();
        let home = child.home.path();
        assert_eq!(shown.ok(), Some(format!("{}||{TOOLCHAIN}|{TARGET}", home.display())));
        let fresh = fs::read_dir(home).is_ok_and(|mut entries| entries.next().is_none());
        assert!(fresh && env::home_dir().is_none_or(|real| real != home) && home.starts_with(fixture.beside()));
    }
}
