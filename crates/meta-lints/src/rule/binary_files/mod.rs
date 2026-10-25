mod module_location;

pub use module_location::ModuleLocation;

use std::collections::{BTreeMap, BTreeSet};

use syn::{Item, ItemMod};

use crate::region::Region;
use crate::source_file::SourceFile;

pub struct BinaryFiles(BTreeSet<String>);

impl BinaryFiles {
    pub fn collect(files: &[SourceFile]) -> BinaryFiles {
        let by_path: BTreeMap<&str, &SourceFile> = files.iter().map(|file| (file.path(), file)).collect();
        BinaryFiles(
            files
                .iter()
                .filter(|file| file.region() == Region::Product && ModuleLocation::from(*file).is_binary_root())
                .fold(BTreeSet::new(), |found, root| Self::gathered(root, &by_path, found)),
        )
    }

    pub fn contains(&self, path: &str) -> bool {
        self.0.contains(path)
    }

    fn gathered(file: &SourceFile, by_path: &BTreeMap<&str, &SourceFile>, found: BTreeSet<String>) -> BTreeSet<String> {
        if found.contains(file.path()) {
            found
        } else {
            let location = ModuleLocation::from(file);
            let with_file: BTreeSet<String> = found.into_iter().chain([file.path().to_owned()]).collect();
            Self::modules(file)
                .flat_map(|module| location.declared(module))
                .filter_map(|candidate| by_path.get(candidate.as_str()).copied())
                .fold(with_file, |found, child| Self::gathered(child, by_path, found))
        }
    }

    fn modules(file: &SourceFile) -> impl Iterator<Item = &ItemMod> {
        file.syntax().items.iter().filter_map(|item| match item {
            Item::Mod(module) => Some(module),
            _ => None,
        })
    }
}
