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

    pub fn and(self, more: Vec<PathBuf>) -> OutsideConfig {
        OutsideConfig(self.0.into_iter().chain(more).collect())
    }

    pub fn refused(self) -> Result<(), Failure> {
        if self.0.is_empty() { Ok(()) } else { Err(Failure::Configured(self)) }
    }
}

impl fmt::Display for OutsideConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "gate refused: config or code outside the commit changes the build:")?;
        self.0.iter().try_for_each(|file| write!(formatter, " {}", file.display()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::OutsideConfig;
    use crate::task::fixture::Fixture;

    const CONFIG: &str = ".cargo/config.toml";
    const HOME_CONFIG: &str = "config.toml";
    const CAPPED: &str = "[env]\nRUSTFLAGS = \"--cap-lints=allow\"\n";
    const ALIASED: &str = "[alias]\nclippy = \"test --no-run\"\n";
    const NETTED: &str = "[net]\nretry = 3\n";
    const HOME: &str = "cargo-home";
    const RUSTFMT: &str = "rustfmt.toml";
    const FORMATTED: &str = "max_width = 200\n";

    impl Fixture {
        fn placed(file: &Path, text: &str) -> PathBuf {
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, text).unwrap();
            file.to_path_buf()
        }

        fn outside(&self) -> OutsideConfig {
            OutsideConfig::around(&self.beside(), &self.path(HOME))
        }
    }

    #[test]
    fn an_ancestor_or_cargo_home_config_that_changes_the_build_is_listed() {
        let fixture = Fixture::new();
        let home = Fixture::placed(&fixture.path(HOME).join(HOME_CONFIG), NETTED);
        assert!(!fixture.outside().0.contains(&home));
        Fixture::placed(&home, ALIASED);
        let ancestor = Fixture::placed(&fixture.beside().with_file_name(CONFIG), CAPPED);
        assert!(fixture.outside().0.contains(&home) && fixture.outside().0.contains(&ancestor));
    }

    #[test]
    fn a_clippy_or_rustfmt_config_above_the_checkout_refuses_the_gate() {
        let fixture = Fixture::new();
        let rustfmt = fixture.beside().with_file_name(RUSTFMT);
        let added = OutsideConfig(Vec::new()).and(vec![rustfmt.clone()]).0 == [rustfmt.clone()];
        assert!(!fixture.outside().0.contains(&rustfmt) && OutsideConfig(Vec::new()).refused().is_ok() && added);
        Fixture::placed(&rustfmt, FORMATTED);
        assert!(fixture.outside().refused().is_err_and(|failure| failure.to_string().contains(RUSTFMT)));
    }
}
