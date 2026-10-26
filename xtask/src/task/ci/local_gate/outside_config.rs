use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::task::ci::local_gate::ConfigEffect;
use crate::task::failure::Failure;

#[derive(Debug)]
pub struct OutsideConfig(Vec<PathBuf>);

impl OutsideConfig {
    const NESTED: [&str; 2] = [".cargo/config.toml", ".cargo/config"];
    const HOME: [&str; 2] = ["config.toml", "config"];
    const TOOLS: [&str; 4] = ["clippy.toml", ".clippy.toml", "rustfmt.toml", ".rustfmt.toml"];

    pub fn around(above: &Path, cargo_home: &Path) -> OutsideConfig {
        let nested = above.ancestors().flat_map(|directory| Self::NESTED.map(|file| directory.join(file)));
        let configs = nested
            .chain(Self::HOME.map(|file| cargo_home.join(file)))
            .filter(|file| fs::read_to_string(file).is_ok_and(|text| text.parse().ok() != Some(ConfigEffect::Neutral)));
        let tools = above.ancestors().flat_map(|directory| Self::TOOLS.map(|file| directory.join(file)));
        OutsideConfig(configs.chain(tools.filter(|file| file.exists())).collect())
    }

    pub fn refused(self) -> Result<(), Failure> {
        if self.0.is_empty() { Ok(()) } else { Err(Failure::Configured(self)) }
    }
}

impl fmt::Display for OutsideConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "gate refused: config outside the commit changes the build:")?;
        self.0.iter().try_for_each(|file| write!(formatter, " {}", file.display()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::OutsideConfig;
    use crate::task::fixture::Fixture;

    const CONFIG: &str = ".cargo/config.toml";
    const CAPPED: &str = "[env]\nRUSTFLAGS = \"--cap-lints=allow\"\n";
    const ALIASED: &str = "[alias]\nxtask = \"run --package xtask --\"\n";
    const HOME: &str = "cargo-home";
    const RUSTFMT: &str = "rustfmt.toml";
    const FORMATTED: &str = "max_width = 200\n";

    #[test]
    fn an_ancestor_or_cargo_home_config_that_changes_the_build_refuses_the_gate() {
        let fixture = Fixture::new();
        let around = || OutsideConfig::around(&fixture.beside(), &fixture.path(HOME));
        fs::create_dir_all(fixture.path(HOME)).unwrap();
        fs::write(fixture.path(HOME).join("config.toml"), ALIASED).unwrap();
        assert!(around().refused().is_ok());
        fixture.write(&format!("../{CONFIG}"), CAPPED);
        assert!(around().refused().is_err_and(|failure| failure.to_string().contains(CONFIG)));
    }

    #[test]
    fn a_clippy_or_rustfmt_config_above_the_checkout_refuses_the_gate() {
        let fixture = Fixture::new();
        let around = || OutsideConfig::around(&fixture.beside(), &fixture.path(HOME));
        assert!(around().refused().is_ok());
        fixture.write(&format!("../{RUSTFMT}"), FORMATTED);
        assert!(around().refused().is_err_and(|failure| failure.to_string().contains(RUSTFMT)));
    }
}
