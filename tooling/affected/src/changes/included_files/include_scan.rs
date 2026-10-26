use std::path::{Component, Path, PathBuf};

use syn::visit::Visit;
use syn::{LitStr, Macro};

use task::SourceFile;

pub struct IncludeScan {
    directory: PathBuf,
    included: Vec<PathBuf>,
}

impl IncludeScan {
    const INCLUDING_MACROS: [&str; 2] = ["include_str", "include_bytes"];

    pub fn in_file(file: &SourceFile) -> Vec<PathBuf> {
        let mut scan = IncludeScan {
            directory: file
                .path()
                .parent()
                .map_or_else(PathBuf::new, Path::to_path_buf),
            included: Vec::new(),
        };
        scan.visit_file(file.syntax());
        scan.included
    }

    fn resolved(&self, literal: &LitStr) -> PathBuf {
        self.directory
            .join(literal.value())
            .components()
            .fold(PathBuf::new(), |path, component| match component {
                Component::ParentDir => path.parent().map(Path::to_path_buf).unwrap_or_default(),
                Component::CurDir => path,
                Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                    path.join(component)
                }
            })
    }
}

impl<'ast> Visit<'ast> for IncludeScan {
    fn visit_macro(&mut self, item: &'ast Macro) {
        self.included.extend(
            item.path
                .get_ident()
                .filter(|ident| Self::INCLUDING_MACROS.iter().any(|name| ident == name))
                .and_then(|_| item.parse_body::<LitStr>().ok())
                .map(|literal| self.resolved(&literal)),
        );
    }
}
