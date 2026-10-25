use std::path::{Path, PathBuf};

use syn::ItemMod;
use syn::visit::Visit;

use crate::rule::name::Name;
use crate::source_file::SourceFile;

#[derive(Default)]
pub(super) struct SupportModules {
    names: Vec<Name>,
}

impl SupportModules {
    const SOURCE_EXTENSION: &str = "rs";
    const MODULE_ROOT: &str = "mod.rs";

    pub(super) fn paths(file: &SourceFile) -> Vec<PathBuf> {
        let mut modules = SupportModules::default();
        modules.visit_file(file.syntax());
        let directory = Self::directory_of(Path::new(file.path()));
        modules
            .names
            .iter()
            .flat_map(|name| {
                [
                    directory.join(name.as_str()).with_extension(Self::SOURCE_EXTENSION),
                    directory.join(name.as_str()).join(Self::MODULE_ROOT),
                ]
            })
            .collect()
    }

    fn directory_of(path: &Path) -> PathBuf {
        match path.file_name().and_then(|name| name.to_str()) {
            Some("mod.rs" | "lib.rs" | "main.rs") => path.parent().map(Path::to_path_buf).unwrap_or_default(),
            _ => path.with_extension(""),
        }
    }
}

impl<'ast> Visit<'ast> for SupportModules {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.names.extend(
            (module.content.is_none() && SourceFile::is_test_item(&module.attrs)).then(|| Name::from(&module.ident)),
        );
        syn::visit::visit_item_mod(self, module);
    }
}
