use std::str::FromStr;

use toml::{Table, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigEffect {
    Neutral,
    Overriding,
}

impl ConfigEffect {
    const ENV: &str = "env";
    const BUILD: &str = "build";
    const TARGET: &str = "target";
    const RUSTFLAGS: &str = "rustflags";
    const BUILD_KEYS: [&str; 4] = ["rustflags", "rustdocflags", "target-dir", "build-dir"];
}

impl FromStr for ConfigEffect {
    type Err = toml::de::Error;

    fn from_str(text: &str) -> Result<ConfigEffect, toml::de::Error> {
        let table: Table = text.parse()?;
        let build = table.get(Self::BUILD).and_then(Value::as_table);
        let targets = table.get(Self::TARGET).and_then(Value::as_table);
        Ok(
            if table.contains_key(Self::ENV)
                || build.is_some_and(|build| Self::BUILD_KEYS.iter().any(|key| build.contains_key(*key)))
                || targets.is_some_and(|targets| {
                    targets.values().filter_map(Value::as_table).any(|target| target.contains_key(Self::RUSTFLAGS))
                })
            {
                ConfigEffect::Overriding
            } else {
                ConfigEffect::Neutral
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigEffect;

    const OVERRIDING: [&str; 3] = [
        "[env]\nRUSTFLAGS = \"--cap-lints=allow\"\n",
        "[build]\nrustflags = [\"--cap-lints=allow\"]\n",
        "[target.aarch64-apple-darwin]\nrustflags = [\"--cap-lints=allow\"]\n",
    ];
    const NEUTRAL: &str = "[alias]\nxtask = \"run --package xtask --\"\n[target.x]\nlinker = \"cc\"\n";

    #[test]
    fn env_build_flags_and_target_flags_override_the_build_and_an_alias_does_not() {
        assert!(OVERRIDING.iter().all(|text| text.parse::<ConfigEffect>().ok() == Some(ConfigEffect::Overriding)));
        assert_eq!(NEUTRAL.parse::<ConfigEffect>().ok(), Some(ConfigEffect::Neutral));
    }
}
