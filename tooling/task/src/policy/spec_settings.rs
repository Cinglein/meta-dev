use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::crate_name::CrateName;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecSettings {
    package: CrateName,
    file: PathBuf,
}

impl SpecSettings {
    pub const UNCONVERTED: &str = "# Not yet converted: hand-written prose, ";
    pub const RULE: &str = "- `";
    pub const CHECK: &str = "check";
    pub const CHECKED: &str = "rung 3, check ";
    pub const ALSO_CHECKED: &str = ", check ";
    pub const WORDING: &str = "): ";

    #[must_use]
    pub fn package(&self) -> &CrateName {
        &self.package
    }

    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }
}

#[cfg(test)]
mod tests {
    use settings::EmbeddedDefaults;

    use crate::policy::Policy;

    const PACKAGE: &str = "meta-spec";
    const FILE: &str = "context/SPEC.md";

    #[test]
    fn the_checked_in_policy_names_its_spec_package_and_file() {
        let policy = Policy::defaults();
        let spec = policy.spec().unwrap();
        assert_eq!(
            (spec.package().to_string().as_str(), spec.file().to_str()),
            (PACKAGE, Some(FILE))
        );
    }
}
