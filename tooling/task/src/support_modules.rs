use std::path::{Path, PathBuf};

use syn::ItemMod;
use syn::visit::Visit;

use crate::name::Name;
use crate::source_file::SourceFile;
use crate::walk::Walk;

#[derive(Debug, Default)]
pub struct SupportModules {
    names: Vec<Name>,
}

impl SupportModules {
    #[must_use]
    pub fn paths(file: &SourceFile) -> Vec<PathBuf> {
        let mut modules = SupportModules::default();
        modules.visit_file(file.syntax());
        let directory = Self::directory_of(file.path());
        modules
            .names
            .iter()
            .flat_map(|name| {
                [
                    directory.join(name).with_extension("rs"),
                    directory.join(name).join("mod.rs"),
                ]
            })
            .collect()
    }

    fn directory_of(path: &Path) -> PathBuf {
        match path.file_name().and_then(|name| name.to_str()) {
            Some("mod.rs" | "lib.rs" | "main.rs") => {
                path.parent().map(Path::to_path_buf).unwrap_or_default()
            }
            _ => path.with_extension(""),
        }
    }
}

impl<'ast> Visit<'ast> for SupportModules {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.names.extend(
            (module.content.is_none() && SourceFile::walk(module) == Walk::Skipped)
                .then(|| Name::from(&module.ident)),
        );
        syn::visit::visit_item_mod(self, module);
    }
}

#[cfg(test)]
mod tests {
    use super::SupportModules;
    use crate::source_file::SourceFile;
    use std::path::PathBuf;
    const SOURCE: &str =
        "#[cfg(test)] mod fixtures; mod inline {} #[cfg(test)] mod tests {} mod sibling;";
    const PATHS: [&str; 2] = ["crates/a/src/fixtures.rs", "crates/a/src/fixtures/mod.rs"];

    #[test]
    fn only_a_declared_cfg_test_module_names_support_files_beside_its_parent() {
        let file = SourceFile::parse("crates/a/src/lib.rs".to_owned(), SOURCE.to_owned())
            .expect("valid rust");
        assert_eq!(SupportModules::paths(&file), PATHS.map(PathBuf::from));
    }
}
