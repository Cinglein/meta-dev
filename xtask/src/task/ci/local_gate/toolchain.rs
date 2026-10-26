use std::ffi::OsString;
use std::fmt;
use std::str::FromStr;

use crate::task::ci::local_gate::Unready;
use crate::task::failure::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toolchain(String);

impl FromStr for Toolchain {
    type Err = Failure;

    fn from_str(shown: &str) -> Result<Toolchain, Failure> {
        shown
            .split_whitespace()
            .next()
            .map(|name| Toolchain(name.to_owned()))
            .ok_or(Failure::Unready(Unready::NoToolchain))
    }
}

impl From<&Toolchain> for OsString {
    fn from(toolchain: &Toolchain) -> OsString {
        OsString::from(&toolchain.0)
    }
}

impl fmt::Display for Toolchain {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::Toolchain;

    const SHOWN: &str = "1.90-x86_64-unknown-linux-gnu (overridden by '/work/rust-toolchain.toml')\n";
    const NAME: &str = "1.90-x86_64-unknown-linux-gnu";

    #[test]
    fn the_active_toolchain_is_the_first_word_rustup_shows() {
        let toolchain: Toolchain = SHOWN.parse().unwrap();
        assert!(toolchain.to_string() == NAME && OsString::from(&toolchain) == NAME);
        assert!("\n".parse::<Toolchain>().is_err());
    }
}
