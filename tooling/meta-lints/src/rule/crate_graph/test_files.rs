use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use syn::{Item, ItemMod};

use crate::rule::crate_graph::name::Name;
use crate::source_file::SourceFile;

pub struct TestFiles(BTreeSet<PathBuf>);

impl TestFiles {
    const MODULE_ROOTS: [&str; 3] = ["main.rs", "mod.rs", "lib.rs"];
    const SOURCE_EXTENSION: &str = "rs";
    const MODULE_ROOT: &str = "mod.rs";
    const CANDIDATE_LENGTH: usize = 2;

    pub fn collect(files: &[SourceFile]) -> TestFiles {
        let by_path: BTreeMap<&Path, &SourceFile> = files.iter().map(|file| (Path::new(file.path()), file)).collect();
        let roots: Vec<&SourceFile> = files
            .iter()
            .flat_map(|file| {
                Self::modules(&file.syntax().items)
                    .into_iter()
                    .filter(|module| SourceFile::is_test_item(&module.attrs) && module.content.is_none())
                    .flat_map(|module| Self::declared_files(file.path(), module))
                    .filter_map(|path| by_path.get(path.as_path()).copied())
                    .collect::<Vec<&SourceFile>>()
            })
            .collect();
        TestFiles(roots.iter().fold(BTreeSet::new(), |found, root| Self::gathered(root, &by_path, found)))
    }

    pub fn contains(&self, path: &str) -> bool {
        self.0.contains(Path::new(path))
    }

    fn gathered(
        file: &SourceFile,
        by_path: &BTreeMap<&Path, &SourceFile>,
        found: BTreeSet<PathBuf>,
    ) -> BTreeSet<PathBuf> {
        if found.contains(Path::new(file.path())) {
            found
        } else {
            let with_file: BTreeSet<PathBuf> = found.into_iter().chain([PathBuf::from(file.path())]).collect();
            Self::modules(&file.syntax().items)
                .into_iter()
                .flat_map(|module| Self::declared_files(file.path(), module))
                .filter_map(|path| by_path.get(path.as_path()).copied())
                .fold(with_file, |found, child| Self::gathered(child, by_path, found))
        }
    }

    fn modules(items: &[Item]) -> Vec<&ItemMod> {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Mod(module) => Some(module),
                _ => None,
            })
            .collect()
    }

    fn declared_files(parent: &str, module: &ItemMod) -> [PathBuf; Self::CANDIDATE_LENGTH] {
        let parent = Path::new(parent);
        let directory = if parent.file_name().is_some_and(|name| Self::MODULE_ROOTS.iter().any(|root| name == *root)) {
            parent.parent().map(Path::to_path_buf)
        } else {
            parent.with_extension("").into()
        }
        .unwrap_or_default();
        let name = Name::from(&module.ident);
        [
            directory.join(name.as_str()).with_extension(Self::SOURCE_EXTENSION),
            directory.join(name.as_str()).join(Self::MODULE_ROOT),
        ]
    }
}
