use std::fmt;

use syn::Path;

use crate::name::Name;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UsePath(Vec<Name>);

impl UsePath {
    const SEPARATOR: &str = "::";

    #[must_use]
    pub fn extended(&self, segment: Name) -> UsePath {
        UsePath(self.0.iter().cloned().chain(Some(segment)).collect())
    }

    #[must_use]
    pub fn rerooted(&self, root: &UsePath) -> UsePath {
        UsePath(
            root.0
                .iter()
                .chain(self.0.iter().skip(1))
                .cloned()
                .collect(),
        )
    }

    #[must_use]
    pub fn first(&self) -> Option<&Name> {
        self.0.first()
    }

    #[must_use]
    pub fn segments(&self) -> &[Name] {
        &self.0
    }
}

impl From<&Path> for UsePath {
    fn from(path: &Path) -> UsePath {
        UsePath(
            path.segments
                .iter()
                .map(|segment| Name::from(&segment.ident))
                .collect(),
        )
    }
}

impl PartialEq<String> for UsePath {
    fn eq(&self, text: &String) -> bool {
        self.0.len() == text.split(Self::SEPARATOR).count()
            && self
                .0
                .iter()
                .zip(text.split(Self::SEPARATOR))
                .all(|(segment, spelled)| *segment == spelled)
    }
}

impl fmt::Display for UsePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spelled: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        formatter.write_str(&spelled.join(Self::SEPARATOR))
    }
}

#[cfg(test)]
mod tests {
    use super::UsePath;
    use crate::name::Name;
    const ALIASED: &str = "Value::Null";
    const FULL: &str = "serde_json::Value";
    const RESOLVED: &str = "serde_json::Value::Null";
    #[test]
    fn a_use_path_prints_with_double_colons_and_rerooting_swaps_its_first_segment_for_a_path() {
        let parsed = |text| UsePath::from(&syn::parse_str::<syn::Path>(text).unwrap());
        let resolved = parsed(ALIASED).rerooted(&parsed(FULL));
        assert!(parsed(ALIASED) == ALIASED.to_owned() && parsed(FULL) != ALIASED.to_owned());
        assert_eq!(resolved.to_string(), RESOLVED);
        let extended = UsePath::default().extended(Name::from("x"));
        assert!(
            resolved.first() == Some(&Name::from("serde_json")) && extended.segments().len() == 1
        );
    }
}
