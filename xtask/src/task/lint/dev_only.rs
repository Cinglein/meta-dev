mod manifest;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use meta_door::Door;
use task::{Policy, Report, SourceFile, Violation};

use manifest::Manifest;

#[derive(Debug)]
pub struct DevOnly;

impl DevOnly {
    pub const LINT: &str = "dev-only";
    const MANIFEST: &str = "Cargo.toml";

    pub fn report(root: &Path, files: &[SourceFile], policy: &Policy) -> Report {
        let paths: BTreeSet<PathBuf> = files
            .iter()
            .filter_map(|file| {
                let crate_dir =
                    file.path().ancestors().skip(1).find(|dir| root.join(dir).join(Self::MANIFEST).is_file());
                crate_dir.map(|dir| dir.join(Self::MANIFEST))
            })
            .collect();
        let manifests: Vec<Manifest> = paths
            .into_iter()
            .filter_map(|path| Door::read(root.join(&path)).ok().and_then(|text| Manifest::parsed(path, &text)))
            .collect();
        Report::new("test-only crates and door features stay in dev-dependencies", Self::found(&manifests, policy))
    }

    fn found(manifests: &[Manifest], policy: &Policy) -> Vec<Violation> {
        let door = |manifest: &&Manifest| manifest.package().is_some_and(|name| policy.file_door().covers(&name));
        let doors: Vec<&Manifest> = manifests.iter().filter(door).collect();
        manifests
            .iter()
            .filter(|manifest| !door(manifest))
            .flat_map(|manifest| manifest.refusals(policy, &doors))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use settings::EmbeddedDefaults;
    use task::Policy;

    use super::{DevOnly, Manifest};

    const DOOR: [&str; 2] = ["door", "[package]\nname = \"meta-door\"\n[features]\nskeleton = []\n"];
    const NORMAL: [&str; 2] = [
        "a",
        "[package]\nname = \"a\"\n[dependencies]\nmeta-door = { workspace = true, features = [\"skeleton\"] }\ntempfile = \"3\"\n",
    ];
    const DEV: [&str; 2] = [
        "b",
        "[package]\nname = \"b\"\n[dev-dependencies]\nmeta-door = { workspace = true, features = [\"skeleton\"] }\ntempfile = \"3\"\n",
    ];
    const SHOWN: [&str; 2] = [
        "a/Cargo.toml: meta-door feature skeleton is test-only; enable it under [dev-dependencies]",
        "a/Cargo.toml: tempfile is test-only; use it under [dev-dependencies] or go through the door",
    ];

    #[test]
    fn a_test_only_crate_or_door_feature_in_normal_dependencies_is_named_and_dev_dependencies_pass() {
        let manifests = [DOOR, NORMAL, DEV]
            .map(|[dir, text]| Manifest::parsed(PathBuf::from(dir).join("Cargo.toml"), text).unwrap());
        let shown: Vec<String> =
            DevOnly::found(&manifests, &Policy::defaults()).iter().map(ToString::to_string).collect();
        assert_eq!(shown, SHOWN);
    }
}
