use std::path::Path;

use task::CrateName;
use task::PackageGraph;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reach {
    Everything,
    Nothing,
    Package(CrateName),
}

impl Reach {
    const ROOT_ENTRIES: [(&str, Reach); 7] = [
        ("Cargo.toml", Reach::Everything),
        ("Cargo.lock", Reach::Everything),
        ("rust-toolchain.toml", Reach::Everything),
        (".cargo/config.toml", Reach::Everything),
        (".github", Reach::Everything),
        (".claude", Reach::Nothing),
        (".gitignore", Reach::Nothing),
    ];
    const INERT_EXTENSIONS: [&str; 1] = ["md"];

    #[must_use]
    pub fn reached_by(path: &Path, packages: &PackageGraph) -> Option<Reach> {
        Self::inert(path)
            .or_else(|| packages.containing(path).cloned().map(Reach::Package))
            .or_else(|| {
                Self::ROOT_ENTRIES
                    .iter()
                    .find(|(entry, _)| path.starts_with(entry))
                    .map(|(_, reach)| reach.clone())
            })
    }

    #[must_use]
    pub fn package(&self) -> Option<CrateName> {
        match self {
            Reach::Package(name) => Some(name.clone()),
            Reach::Everything | Reach::Nothing => None,
        }
    }

    fn inert(path: &Path) -> Option<Reach> {
        path.extension()
            .filter(|extension| {
                Self::INERT_EXTENSIONS
                    .iter()
                    .any(|inert| extension == inert)
            })
            .map(|_| Reach::Nothing)
    }
}

#[cfg(test)]
mod tests {
    use super::Reach;
    use std::path::Path;
    use task::PackageGraph;
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "eval", "manifest_path": "/w/crates/eval/Cargo.toml", "dependencies": []}
    ]}"#;
    const PACKAGE_PATH: &str = "crates/eval/src/score.rs";
    const EVAL: &str = "eval";
    const EVERYTHING_PATHS: [&str; 2] = ["Cargo.lock", ".github/workflows/ci.yml"];
    const NOTHING_PATHS: [&str; 2] = ["README.md", ".claude/settings.json"];
    const UNLISTED_ROOT_FILE: &str = "tooling/policy.toml";

    #[test]
    fn a_path_reaches_its_package_a_root_file_its_entry_and_an_unlisted_root_file_waits() {
        let packages = PackageGraph::from_metadata(METADATA).unwrap();
        let reach = |path: &str| Reach::reached_by(Path::new(path), &packages);
        assert_eq!(
            reach(PACKAGE_PATH).and_then(|reach| reach.package()),
            EVAL.parse().ok()
        );
        assert!(
            EVERYTHING_PATHS
                .iter()
                .all(|path| reach(path) == Some(Reach::Everything))
                && NOTHING_PATHS
                    .iter()
                    .all(|path| reach(path) == Some(Reach::Nothing))
        );
        assert!(reach(UNLISTED_ROOT_FILE).is_none());
    }
}
