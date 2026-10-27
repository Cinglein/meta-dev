use std::convert::Infallible;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved(Vec<Option<String>>);

impl Resolved {
    pub fn objects(&self) -> &[Option<String>] {
        &self.0
    }
}

impl FromStr for Resolved {
    type Err = Infallible;

    fn from_str(listed: &str) -> Result<Self, Self::Err> {
        let object = |line: &str| {
            (!line.is_empty() && line.chars().all(|letter| letter.is_ascii_hexdigit())).then(|| line.to_owned())
        };
        Ok(Self(listed.lines().map(object).collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::Resolved;

    #[test]
    fn a_hex_line_is_an_object_and_a_missing_name_or_a_blank_line_is_none() {
        let Ok(resolved) = "c0ffee\nORIG_HEAD missing\n\nx1\n".parse::<Resolved>();
        assert_eq!(resolved.objects(), [Some("c0ffee".to_owned()), None, None, None]);
    }
}
