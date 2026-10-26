use std::collections::BTreeMap;

use serde::Deserialize;

use task::CrateName;
use task::Failure;
use task::MutantCount;

#[derive(Debug, Deserialize)]
pub struct ListedMutant {
    package: CrateName,
}

impl ListedMutant {
    pub fn tally(json: &str) -> Result<BTreeMap<CrateName, MutantCount>, Failure> {
        let listed: Vec<ListedMutant> = if json.bytes().all(|byte| byte.is_ascii_whitespace()) {
            Vec::new()
        } else {
            serde_json::from_str(json).map_err(Failure::MutantList)?
        };
        Ok(listed
            .into_iter()
            .fold(BTreeMap::new(), |mut counts, mutant| {
                *counts.entry(mutant.package).or_default() += MutantCount::from(1);
                counts
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::{ListedMutant, MutantCount};
    const LISTED: &str = r#"[{"package": "log", "file": "crates/log/src/lib.rs"},
        {"package": "state"}, {"package": "log"}]"#;
    const BOARD: usize = 2;

    #[test]
    fn a_mutant_list_is_tallied_per_package_whatever_else_each_entry_carries() {
        let counts = ListedMutant::tally(LISTED).unwrap();
        assert_eq!(
            (counts[&"log".parse().unwrap()], counts.len()),
            (MutantCount::from(BOARD), BOARD)
        );
        assert!(ListedMutant::tally("{").is_err() && ListedMutant::tally("\n").unwrap().is_empty());
    }
}
