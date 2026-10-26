use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;

use crate::task::ci::local_gate::MetadataPackage;

#[derive(Debug, Deserialize)]
pub struct CargoMetadata {
    packages: Vec<MetadataPackage>,
}

impl CargoMetadata {
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

    use super::CargoMetadata;

    const METADATA: &str = r#"{"packages": [
        {"name": "app", "manifest_path": "/gate/checkout/app/Cargo.toml", "source": null},
        {"name": "vendored", "manifest_path": "/elsewhere/vendored/Cargo.toml", "source": null},
        {"name": "serde", "manifest_path": "/cargo/registry/serde/Cargo.toml", "source": "registry+https://example.invalid"}
    ]}"#;
    const CHECKOUT: &str = "/gate/checkout";
    const OUTSIDE: &str = "/elsewhere/vendored/Cargo.toml";

    #[test]
    fn only_local_packages_outside_the_checkout_are_listed() {
        let metadata: CargoMetadata = METADATA.parse().unwrap();
        let outside = metadata.outside(|manifest| manifest.starts_with(CHECKOUT));
        assert_eq!(outside, [PathBuf::from(OUTSIDE)]);
        assert!(metadata.outside(|_| true).is_empty() && metadata.outside(|_| false).len() == 2);
        assert!(Path::new(OUTSIDE).is_absolute());
    }
}
