use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait EmbeddedDefaults: Serialize + DeserializeOwned {
    const DEFAULTS: &str;

    #[must_use]
    #[expect(clippy::expect_used, reason = "the embedded defaults are checked in and parsed by a test")]
    fn defaults() -> Self {
        toml::from_str(Self::DEFAULTS).expect("the embedded default settings parse")
    }
}

#[cfg(test)]
mod tests {
    use super::EmbeddedDefaults;
    use serde::{Deserialize, Serialize};
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Knobs {
        rounds: usize,
    }

    impl EmbeddedDefaults for Knobs {
        const DEFAULTS: &str = "rounds = 3";
    }

    #[test]
    fn the_embedded_defaults_are_read_into_the_type() {
        assert_eq!(Knobs::defaults(), Knobs { rounds: 3 });
    }
}
