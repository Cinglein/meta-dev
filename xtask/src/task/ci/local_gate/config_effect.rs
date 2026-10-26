use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;

use toml::{Table, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigEffect {
    Neutral,
    Overriding,
}

impl ConfigEffect {
    const NEUTRAL_TABLES: [&str; 5] = ["alias", "net", "http", "registries", "term"];
    const BUILD: &str = "build";
    const JOBS: &str = "jobs";
    const WRAPPER: &str = "rustc-wrapper";
    const SCCACHE: &str = "sccache";
}

impl FromStr for ConfigEffect {
    type Err = toml::de::Error;

    fn from_str(text: &str) -> Result<ConfigEffect, toml::de::Error> {
        let table: Table = text.parse()?;
        let sccache = |wrapper: &str| Path::new(wrapper).file_name() == Some(OsStr::new(Self::SCCACHE));
        let neutral_build = |build: &Table| {
            build
                .iter()
                .all(|(key, value)| key == Self::JOBS || (key == Self::WRAPPER && value.as_str().is_some_and(sccache)))
        };
        let neutral = |(key, value): (&String, &Value)| {
            Self::NEUTRAL_TABLES.contains(&key.as_str())
                || (key == Self::BUILD && value.as_table().is_some_and(neutral_build))
        };
        Ok(if table.iter().all(neutral) { ConfigEffect::Neutral } else { ConfigEffect::Overriding })
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigEffect;

    const OVERRIDING: [&str; 6] = [
        "[target.aarch64-apple-darwin]\nrunner = \"true\"\n",
        "[profile.dev]\ndebug-assertions = false\n",
        "[build]\nrustc-wrapper = \"/tmp/wrap.sh\"\n",
        "[env]\nRUSTFLAGS = \"--cap-lints=allow\"\n",
        "[build]\njobs = 2\nrustflags = [\"--cap-lints=allow\"]\n",
        "[build]\nrustc-wrapper = \"/tmp/sccache-not\"\n",
    ];
    const NEUTRAL: [&str; 2] = [
        "[alias]\nxtask = \"run --package xtask --\"\n[net]\nretry = 3\n[term]\nverbose = false\n",
        "[build]\njobs = 4\nrustc-wrapper = \"/opt/homebrew/bin/sccache\"\n[http]\ntimeout = 30\n[registries]\n",
    ];

    #[test]
    fn only_listed_keys_and_an_sccache_wrapper_leave_the_build_alone() {
        assert!(OVERRIDING.iter().all(|text| text.parse::<ConfigEffect>().ok() == Some(ConfigEffect::Overriding)));
        assert!(NEUTRAL.iter().all(|text| text.parse::<ConfigEffect>().ok() == Some(ConfigEffect::Neutral)));
    }
}
