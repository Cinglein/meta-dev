mod metadata_package;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;
use task::{CargoArgument, CommandLine, Policy, Program, Stdout, Violation};

pub use metadata_package::MetadataPackage;

#[derive(Debug, Deserialize)]
pub struct CargoMetadata {
    packages: Vec<MetadataPackage>,
}

impl CargoMetadata {
    const ARGUMENTS: [CargoArgument; 3] =
        [CargoArgument::Metadata, CargoArgument::FormatVersionOne, CargoArgument::NoDeps];

    pub fn read(root: &Path) -> Result<CargoMetadata, task::Failure> {
        let command = CommandLine::new(Program::Cargo).in_directory(root.to_path_buf());
        let json = Self::ARGUMENTS.into_iter().fold(command, CommandLine::argument).captured::<Stdout>()?;
        Ok(json.parse::<CargoMetadata>()?)
    }

    pub fn dev_only(&self, policy: &Policy, touched: impl Fn(&MetadataPackage) -> bool) -> Vec<Violation> {
        let door = |package: &&MetadataPackage| policy.file_door().covers(package.name());
        let doors: Vec<&MetadataPackage> = self.packages.iter().filter(door).collect();
        let checked = self.packages.iter().filter(|package| !door(package) && touched(package));
        checked.flat_map(|package| package.refusals(policy, &doors)).collect()
    }

    pub fn outside(&self, inside: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
        self.packages
            .iter()
            .filter_map(MetadataPackage::local_manifest)
            .filter(|manifest| !inside(manifest))
            .map(Path::to_path_buf)
            .collect()
    }
}

impl FromStr for CargoMetadata {
    type Err = serde_json::Error;

    fn from_str(json: &str) -> Result<CargoMetadata, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use settings::EmbeddedDefaults;
    use task::Policy;

    use super::CargoMetadata;

    const METADATA: &str = r#"{"packages": [
        {"name": "app", "manifest_path": "/gate/checkout/app/Cargo.toml", "source": null},
        {"name": "vendored", "manifest_path": "/elsewhere/vendored/Cargo.toml", "source": null},
        {"name": "serde", "manifest_path": "/cargo/registry/serde/Cargo.toml", "source": "registry+https://example.invalid"}
    ]}"#;
    const CHECKOUT: &str = "/gate/checkout";
    const OUTSIDE: &str = "/elsewhere/vendored/Cargo.toml";
    const DEPENDING: &str = r#"{"packages": [
        {"name": "meta-door", "manifest_path": "/w/door/Cargo.toml", "source": null, "dependencies": [],
         "features": {"default": [], "skeleton": []}},
        {"name": "a", "manifest_path": "/w/a/Cargo.toml", "source": null, "features": {"fixtures": ["door?/skeleton"]},
         "dependencies": [{"name": "meta-door", "rename": "door", "kind": null, "features": []}]},
        {"name": "b", "manifest_path": "/w/b/Cargo.toml", "source": null, "features": {"fixtures": ["meta-door/skeleton"]},
         "dependencies": [{"name": "meta-door", "rename": null, "kind": "dev", "features": ["skeleton"]},
            {"name": "tempfile", "rename": null, "kind": "dev", "features": []}]},
        {"name": "c", "manifest_path": "/w/c/Cargo.toml", "source": null,
         "dependencies": [{"name": "meta-door", "rename": null, "kind": "build", "features": ["skeleton"]}]},
        {"name": "d", "manifest_path": "/w/d/Cargo.toml", "source": null,
         "dependencies": [{"name": "tempfile", "rename": "scratch", "kind": null, "features": [], "target": "cfg(unix)"}]}
    ]}"#;
    const REFUSED: [&str; 3] = [
        "/w/a/Cargo.toml: meta-door feature skeleton is test-only; enable it under [dev-dependencies]",
        "/w/c/Cargo.toml: meta-door feature skeleton is test-only; enable it under [dev-dependencies]",
        "/w/d/Cargo.toml: tempfile is test-only; use it under [dev-dependencies] or go through the door",
    ];

    #[test]
    fn renamed_target_build_and_forwarded_test_only_uses_are_named_and_dev_dependencies_pass() {
        let metadata: CargoMetadata = DEPENDING.parse().unwrap();
        let found = metadata.dev_only(&Policy::defaults(), |_| true);
        assert_eq!(found.iter().map(ToString::to_string).collect::<Vec<_>>(), REFUSED);
    }

    #[test]
    fn only_local_packages_outside_the_checkout_are_listed() {
        let metadata: CargoMetadata = METADATA.parse().unwrap();
        let outside = metadata.outside(|manifest| manifest.starts_with(CHECKOUT));
        assert_eq!(outside, [PathBuf::from(OUTSIDE)]);
        assert!(metadata.outside(|_| true).is_empty() && metadata.outside(|_| false).len() == 2);
        assert!(Path::new(OUTSIDE).is_absolute());
    }
}
