use std::fmt::Display;
use std::fs;
use std::num::NonZeroU32;
use std::path::Path;
use std::str::FromStr;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::reading::Reading;

#[derive(Debug)]
pub struct Laws;

impl Laws {
    const POLICY: &str = "tooling/policy.toml";
    const LIMITS: &str = "limits";
    const CASES: &str = "law_cases";

    #[must_use]
    pub fn cases(within: &Path) -> Option<NonZeroU32> {
        let text = within.ancestors().find_map(|folder| fs::read_to_string(folder.join(Self::POLICY)).ok())?;
        let policy: toml::Table = toml::from_str(&text).ok()?;
        let cases = policy.get(Self::LIMITS)?.get(Self::CASES)?.as_integer()?;
        u32::try_from(cases).ok().and_then(NonZeroU32::new)
    }

    pub const NONE_MISREAD: [&str; 0] = [];

    #[must_use]
    pub fn misread<'table, Kind>(
        accepted: &[Reading<'table>],
        refused: &[&'table str],
        refusal: impl Fn(&str) -> Kind::Err,
    ) -> Vec<&'table str>
    where
        Kind: FromStr + Display,
        Kind::Err: PartialEq,
    {
        let accepted = accepted.iter().filter(|reading| reading.misread::<Kind>()).map(|reading| reading.text());
        let refused = refused.iter().filter(|text| text.parse::<Kind>().err() != Some(refusal(text)));
        accepted.chain(refused.copied()).collect()
    }

    #[must_use]
    pub fn shown<Kind: FromStr + Display + PartialEq>(value: &Kind) -> bool {
        ToString::to_string(value).parse::<Kind>().is_ok_and(|read| read == *value)
    }

    #[must_use]
    pub fn settled<Kind: FromStr + Display + PartialEq>(text: &str) -> bool {
        text.parse::<Kind>().is_ok_and(|value| Self::shown(&value))
    }

    #[must_use]
    pub fn stored<Kind: FromStr + Serialize + DeserializeOwned + PartialEq>(text: &str) -> bool {
        text.parse::<Kind>().is_ok_and(|value| Self::recorded(&value))
    }

    #[must_use]
    pub fn recorded<Kind: Serialize + DeserializeOwned + PartialEq>(value: &Kind) -> bool {
        let read = serde_json::to_string(value).and_then(|stored| serde_json::from_str::<Kind>(&stored));
        read.is_ok_and(|read| read == *value)
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;
    use std::path::Path;

    use super::Laws;
    use crate::Reading;

    const UNEQUAL: &str = "NaN";
    const ACCEPTED: [Reading; 2] = [Reading::new("07", "7"), Reading::verbatim("x")];
    const REFUSED: [&str; 3] = ["", "x", "8"];
    const MISREAD: [&str; 3] = ["x", "x", "8"];

    #[test]
    fn a_value_unequal_to_itself_breaks_every_law_and_a_law_runs_more_than_one_case() {
        assert!(!Laws::shown(&f64::NAN) && !Laws::settled::<f64>(UNEQUAL));
        assert!(!Laws::stored::<f64>(UNEQUAL) && !Laws::recorded(&f64::NAN));
        assert!(Laws::cases(Path::new(env!("CARGO_MANIFEST_DIR"))) > Some(NonZeroU32::MIN));
    }

    #[test]
    fn a_table_names_each_input_read_otherwise_than_it_says() {
        let empty = |_: &str| "".parse::<u8>().unwrap_err();
        assert_eq!(Laws::misread::<u8>(&ACCEPTED, &REFUSED, empty), MISREAD);
    }
}
