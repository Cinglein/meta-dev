use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sha(String);

impl Sha {
    const HEX_LENGTH_PER_BYTE: usize = 2;

    pub(crate) fn digest_length(&self) -> usize {
        self.0.len() / Self::HEX_LENGTH_PER_BYTE
    }
}

impl FromStr for Sha {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(text.trim().to_owned()))
    }
}

impl fmt::Display for Sha {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Sha;

    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn only_a_forty_hex_commit_id_parses() {
        let refused =
            ["", "HEAD", "origin/meta/log", "-p", &COMMIT[1..], &format!("{COMMIT}0"), &COMMIT.replace('0', "g")];
        assert!(refused.iter().all(|text| text.parse::<Sha>().is_err()));
        assert_eq!(format!("{COMMIT}\n").parse::<Sha>().map(|sha| sha.to_string()).ok().as_deref(), Some(COMMIT));
    }
}
