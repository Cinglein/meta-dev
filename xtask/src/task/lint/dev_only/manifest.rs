mod dev_only_finding;

use std::collections::BTreeSet;
use std::path::PathBuf;

use task::{CrateName, Policy, Site, Violation};
use toml::{Table, Value};

use dev_only_finding::DevOnlyFinding;

#[derive(Debug)]
pub struct Manifest {
    path: PathBuf,
    table: Table,
}

impl Manifest {
    const PACKAGE: &str = "package";
    const NAME: &str = "name";
    const DEPENDENCIES: &str = "dependencies";
    const FEATURES: &str = "features";

    pub fn parsed(path: PathBuf, text: &str) -> Option<Manifest> {
        text.parse::<Table>().ok().map(|table| Manifest { path, table })
    }

    pub fn package(&self) -> Option<CrateName> {
        let package = self.table.get(Self::PACKAGE).and_then(Value::as_table)?;
        package.get(Self::NAME).and_then(Value::as_str).and_then(|name| name.parse().ok())
    }

    pub fn refusals(&self, policy: &Policy, doors: &[&Manifest]) -> Vec<Violation> {
        let dependencies = self.table.get(Self::DEPENDENCIES).and_then(Value::as_table);
        dependencies
            .into_iter()
            .flatten()
            .filter_map(|(dependency, spec)| {
                let crate_name: CrateName = dependency.parse().ok()?;
                let enabled = Self::named(spec.get(Self::FEATURES).and_then(Value::as_array).into_iter().flatten());
                let door = doors.iter().find(|door| door.package().as_ref() == Some(&crate_name));
                let gated = door.and_then(|door| door.gates().intersection(&enabled).next().cloned());
                match (policy.file_door().dev_only().contains(&crate_name), gated) {
                    (true, _) => Some(DevOnlyFinding::Dependency(crate_name)),
                    (false, Some(feature)) => Some(DevOnlyFinding::Feature(crate_name, feature)),
                    (false, None) => None,
                }
            })
            .map(|finding| Violation::new(Site::File(self.path.clone()), finding))
            .collect()
    }

    fn gates(&self) -> BTreeSet<String> {
        let features = self.table.get(Self::FEATURES).and_then(Value::as_table);
        features.into_iter().flat_map(|features| features.keys().cloned()).collect()
    }

    fn named<'value>(values: impl Iterator<Item = &'value Value>) -> BTreeSet<String> {
        values.filter_map(Value::as_str).map(str::to_owned).collect()
    }
}
