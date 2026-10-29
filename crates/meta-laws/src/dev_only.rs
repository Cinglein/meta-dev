use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const MANIFEST: &str = "Cargo.toml";
const LAWS: &str = env!("CARGO_PKG_NAME");
const SECTIONS: [&str; 3] = ["dev-dependencies", "dependencies", "build-dependencies"];

struct Manifests;

impl Manifests {
    fn read(path: &Path) -> toml::Table {
        toml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    fn matching(root: &Path, pattern: &str) -> Vec<PathBuf> {
        pattern.strip_suffix("/*").map_or_else(
            || vec![root.join(pattern)],
            |parent| fs::read_dir(root.join(parent)).unwrap().map(|entry| entry.unwrap().path()).collect(),
        )
    }

    fn members() -> Vec<PathBuf> {
        let root = Path::new(ROOT);
        let workspace = Self::read(&root.join(MANIFEST));
        let patterns = workspace["workspace"]["members"].as_array().unwrap().clone();
        let folders = patterns.iter().flat_map(|pattern| Self::matching(root, pattern.as_str().unwrap()));
        folders.map(|folder| folder.join(MANIFEST)).filter(|manifest| manifest.is_file()).collect()
    }

    fn naming_the_laws(section: &str) -> Vec<PathBuf> {
        let names =
            |manifest: &PathBuf| Self::read(manifest).get(section).is_some_and(|table| table.get(LAWS).is_some());
        Self::members().into_iter().filter(names).collect()
    }
}

#[test]
fn only_dev_dependencies_name_the_laws_so_no_shipped_code_links_them() {
    let [dev, shipped, built] = SECTIONS.map(Manifests::naming_the_laws);
    assert_ne!(dev, Vec::<PathBuf>::new());
    assert_eq!([shipped, built], [Vec::<PathBuf>::new(), Vec::new()]);
}
