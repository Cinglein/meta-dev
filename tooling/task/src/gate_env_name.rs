use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GateEnvName(String);

impl GateEnvName {
    const EXACT: [&str; 4] = ["HOME", "PATH", "RUSTFLAGS", "CARGO"];
    const PREFIXES: [&str; 10] = [
        "CARGO_", "RUSTC", "RUSTUP_", "RUSTDOC", "RUST_", "RUSTFMT", "CLIPPY", "XDG_", "LD_",
        "DYLD_",
    ];

    fn changes_build(upper: &str) -> bool {
        Self::EXACT.contains(&upper)
            || Self::PREFIXES
                .iter()
                .any(|prefix| upper.starts_with(prefix))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for GateEnvName {
    type Error = String;

    fn try_from(name: String) -> Result<GateEnvName, String> {
        match name.to_ascii_uppercase() {
            _ if name.is_empty()
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') =>
            {
                Err(format!(
                    "gate_env name {name:?} is not an environment variable name"
                ))
            }
            upper if Self::changes_build(&upper) => {
                Err(format!("gate_env name {name} could change the build"))
            }
            _ => Ok(GateEnvName(name)),
        }
    }
}

impl From<GateEnvName> for String {
    fn from(name: GateEnvName) -> String {
        name.0
    }
}

impl fmt::Display for GateEnvName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use settings::EmbeddedDefaults;

    use super::GateEnvName;
    use crate::policy::Policy;

    const REFUSED: [&str; 24] = [
        "RUSTFLAGS", "CARGO_HOME", "CARGO_BUILD_RUSTFLAGS", "RUSTC", "RUSTC_WRAPPER",
        "RUSTUP_TOOLCHAIN", "RUSTDOCFLAGS", "HOME", "XDG_CONFIG_HOME", "PATH", "LD_PRELOAD",
        "DYLD_LIBRARY_PATH", "rustflags", "", "A=B", "A B", "RUST_LOG", "RUST_BACKTRACE", "CARGO",
        "cargo", "RUSTFMT", "RUSTFMT_CONFIG", "CLIPPY_CONF_DIR", "CLIPPY",
    ];
    const ACCEPTED: [&str; 6] = [
        "CHROME", "HOMEBREW_PREFIX", "PATHEXT", "RUSTY", "CARGOX", "MYCARGO",
    ];

    #[test]
    fn a_name_that_could_change_the_build_or_is_not_a_name_is_refused() {
        let kept: Vec<&str> = REFUSED
            .into_iter()
            .filter(|name| GateEnvName::try_from(name.to_string()).is_ok())
            .collect();
        assert!(kept.is_empty(), "{kept:?}");
        assert!(
            ACCEPTED
                .into_iter()
                .all(|name| GateEnvName::try_from(name.to_string()).is_ok())
        );
    }

    #[test]
    fn the_policy_refuses_such_a_name_when_read_and_lists_the_others_in_order() {
        let policy_with = |names: &[&str]| {
            toml::from_str::<Policy>(&format!("gate_env = {names:?}\n{}", Policy::DEFAULTS))
        };
        let named = policy_with(&ACCEPTED).unwrap();
        let shown: Vec<&str> = named.gate_env().iter().map(GateEnvName::as_str).collect();
        assert_eq!(shown, ACCEPTED);
        assert!(
            policy_with(&[ACCEPTED[0], REFUSED[0]]).is_err()
                && Policy::defaults().gate_env().is_empty()
        );
        assert_eq!(
            toml::from_str::<Policy>(&toml::to_string(&named).unwrap()).unwrap(),
            named
        );
    }
}
