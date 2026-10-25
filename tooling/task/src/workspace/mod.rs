mod package_graph;

use std::collections::BTreeMap;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use crate::command_line::{CargoArgument, CommandLine, Program, Stdout};
use crate::crate_name::CrateName;
use crate::failure::Failure;
use crate::site::Site;
use crate::source_file::SourceFile;
use package_graph::PackageGraph;

#[derive(Debug)]
pub struct Workspace {
    root: PathBuf,
    packages: PackageGraph,
}

impl Workspace {
    const MANIFEST: &str = "Cargo.toml";
    const WORKSPACE_TABLE: &str = "[workspace]";

    pub fn locate() -> Result<Workspace, Failure> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .find(|directory| {
                fs::read_to_string(directory.join(Self::MANIFEST))
                    .is_ok_and(|manifest| manifest.contains(Self::WORKSPACE_TABLE))
            })
            .ok_or(Failure::NoWorkspace)?
            .to_path_buf();
        let metadata = CommandLine::new(Program::Cargo)
            .in_directory(root.clone())
            .argument(CargoArgument::Metadata)
            .argument(CargoArgument::FormatVersionOne)
            .argument(CargoArgument::NoDeps)
            .captured::<Stdout>()?;
        Ok(Workspace {
            root,
            packages: PackageGraph::from_metadata(&metadata)?,
        })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn line(&self, program: Program) -> CommandLine {
        CommandLine::new(program).in_directory(self.root.clone())
    }

    pub fn source_files(&self) -> Result<Vec<SourceFile>, Failure> {
        let located = self.packages.directories().iter().try_fold(
            BTreeMap::new(),
            |mut located: BTreeMap<PathBuf, &CrateName>, (name, directory)| {
                located.extend(
                    Self::rust_files_under(&self.root.join(directory))?
                        .into_iter()
                        .map(|file| (file, name)),
                );
                Ok::<BTreeMap<PathBuf, &CrateName>, Failure>(located)
            },
        )?;
        located
            .iter()
            .map(|(file, name)| SourceFile::read(self, file, (*name).clone()))
            .collect()
    }

    #[must_use]
    pub fn relative(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.root).unwrap_or(path).to_path_buf()
    }

    fn rust_files_under(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let io = |error| Failure::Io {
            site: Site::File(directory.to_path_buf()),
            error,
        };
        fs::read_dir(directory)
            .map_err(io)?
            .map(|entry| {
                entry
                    .map_err(io)
                    .and_then(|entry| Self::rust_files_in(&entry))
            })
            .collect::<Result<Vec<Vec<PathBuf>>, Failure>>()
            .map(|nested| nested.into_iter().flatten().collect())
    }

    fn rust_files_in(entry: &DirEntry) -> Result<Vec<PathBuf>, Failure> {
        let path = entry.path();
        if path.is_dir() {
            Self::rust_files_under(&path)
        } else {
            Ok(path
                .extension()
                .is_some_and(|extension| extension == "rs")
                .then_some(path)
                .into_iter()
                .collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PackageGraph, Workspace};
    use std::fs;
    const TREE: [&str; 4] = ["a/src/a.rs", "b/b.rs", "target/skipped.rs", "a/notes.txt"];
    const FOUND: [&str; 2] = ["a:a/src/a.rs", "b:b/b.rs"];
    const TEXT: &str = "pub struct Found;";
    const ROOT_MARK: &str = "ROOT";
    const METADATA: &str = r#"{"workspace_root": "ROOT", "packages": [
        {"name": "a", "manifest_path": "ROOT/a/Cargo.toml", "dependencies": []},
        {"name": "b", "manifest_path": "ROOT/b/Cargo.toml", "dependencies": []}
    ]}"#;

    #[test]
    fn rust_files_are_found_below_the_members_only_and_named_relative_to_the_root() {
        let root = std::env::temp_dir().join(format!("tooling-workspace-{}", std::process::id()));
        TREE.iter().map(|file| root.join(file)).for_each(|path| {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, TEXT).unwrap();
        });
        let metadata = METADATA.replace(ROOT_MARK, &root.display().to_string());
        let packages = PackageGraph::from_metadata(&metadata).unwrap();
        let workspace = Workspace { root, packages };
        let files = workspace.source_files().unwrap();
        let found = files
            .iter()
            .map(|file| format!("{}:{}", file.crate_name().unwrap(), file.path().display()));
        assert!(found.eq(FOUND.map(String::from)));
        fs::remove_dir_all(workspace.root()).unwrap();
    }
}
