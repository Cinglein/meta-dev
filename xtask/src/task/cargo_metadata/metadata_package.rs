mod declared;
mod dev_only_finding;
mod forwarded;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::IgnoredAny;
use task::{CrateName, Policy, Site, Violation};

use declared::Declared;
use dev_only_finding::DevOnlyFinding;
use forwarded::Forwarded;

#[derive(Debug, Deserialize)]
pub struct MetadataPackage {
    name: CrateName,
    manifest_path: PathBuf,
    source: Option<IgnoredAny>,
    #[serde(default)]
    dependencies: Vec<Declared>,
    #[serde(default)]
    features: BTreeMap<String, Vec<Forwarded>>,
}

impl MetadataPackage {
    const DEFAULT: &str = "default";

    pub fn local_manifest(&self) -> Option<&Path> {
        self.source.is_none().then_some(self.manifest_path.as_path())
    }

    pub fn name(&self) -> &CrateName {
        &self.name
    }

    pub fn refusals(&self, policy: &Policy, doors: &[&MetadataPackage]) -> Vec<Violation> {
        let built = self.dependencies.iter().filter(|declared| declared.built());
        let found: BTreeSet<DevOnlyFinding> =
            built.flat_map(|declared| self.refused(declared, policy, doors)).collect();
        found.into_iter().map(|finding| Violation::new(Site::File(self.manifest_path.clone()), finding)).collect()
    }

    fn refused(&self, declared: &Declared, policy: &Policy, doors: &[&MetadataPackage]) -> Vec<DevOnlyFinding> {
        let name = declared.name();
        let listed = policy.file_door().dev_only().contains(name).then(|| DevOnlyFinding::Dependency(name.clone()));
        let alias = declared.rename().unwrap_or(name);
        let forwarded = self.features.values().flatten().filter_map(|value| value.enabling(alias));
        let enabled: BTreeSet<String> = declared.features().iter().cloned().chain(forwarded).collect();
        let door = doors.iter().find(|door| door.name() == name);
        let gated = door.map(|door| door.gates(enabled));
        let features = gated.into_iter().flatten().map(|feature| DevOnlyFinding::Feature(name.clone(), feature));
        listed.into_iter().chain(features).collect()
    }

    fn gates(&self, enabled: BTreeSet<String>) -> Vec<String> {
        let defaults = self.features.get(Self::DEFAULT).into_iter().flatten().filter_map(Forwarded::local);
        let enabled: BTreeSet<String> = enabled.into_iter().chain(defaults).collect();
        let gates = self.features.keys().filter(|feature| *feature != Self::DEFAULT);
        gates.filter(|feature| enabled.contains(*feature)).cloned().collect()
    }
}
