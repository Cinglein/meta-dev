use std::path::Path;

use serde::Deserialize;
use task::SpecSettings;

use crate::task::failure::Failure;

#[derive(Debug, Deserialize)]
pub struct PriorSpec {
    spec: Option<SpecSettings>,
}

impl PriorSpec {
    pub fn prose(policy: Option<&str>, read: impl Fn(&Path) -> Option<String>) -> Result<Option<String>, Failure> {
        let prior = policy.map(toml::from_str::<PriorSpec>).transpose().map_err(Failure::PriorSpec)?;
        prior
            .and_then(|prior| prior.spec)
            .map(|spec| read(spec.file()).ok_or_else(|| Failure::PriorSpecFile(spec.file().to_path_buf())))
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::PriorSpec;

    const RENAMED: &str = "vague = []\n[spec]\npackage = \"spec\"\npath = \"moved/SPEC.md\"\n";
    const PLACED: &str = "vague = []\n[spec]\npackage = \"spec\"\nfile = \"context/SPEC.md\"\n";
    const UNSET: &str = "vague = []\n";
    const PROSE: &str = "# Spec\n";

    #[test]
    fn a_merge_base_spec_that_does_not_parse_or_cannot_be_read_fails_and_an_absent_one_sets_no_limit() {
        let readable = |path: &Path| (path == Path::new("context/SPEC.md")).then(|| PROSE.to_owned());
        assert!(
            PriorSpec::prose(Some(RENAMED), readable).is_err() && PriorSpec::prose(Some(PLACED), |_| None).is_err()
        );
        assert_eq!(PriorSpec::prose(Some(PLACED), readable).ok().flatten().as_deref(), Some(PROSE));
        assert!(
            PriorSpec::prose(Some(UNSET), readable).is_ok_and(|prose| prose.is_none())
                && PriorSpec::prose(None, readable).is_ok_and(|prose| prose.is_none())
        );
    }
}
