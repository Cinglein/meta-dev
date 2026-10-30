use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ExprPath, Ident};

use task::{LineNumber, SourceFile};

#[derive(Debug, Default)]
pub(crate) struct AmbientPaths {
    found: Vec<LineNumber>,
}

impl AmbientPaths {
    const ENV: &str = "env";
    const IO: &str = "io";
    const STDIN: &str = "stdin";
    const READS: [&str; 9] = [
        "var", "var_os", "vars", "vars_os", "args", "args_os", "current_dir", "current_exe",
        "temp_dir",
    ];

    pub(crate) fn found(file: &SourceFile) -> Vec<LineNumber> {
        let mut paths = AmbientPaths::default();
        paths.visit_file(file.syntax());
        paths.found
    }

    fn ambient(names: &[&Ident]) -> Option<()> {
        match names {
            [.., module, read]
                if *module == Self::ENV && Self::READS.iter().any(|name| *read == name) =>
            {
                Some(())
            }
            [.., module, read] if *module == Self::IO && *read == Self::STDIN => Some(()),
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for AmbientPaths {
    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        let names: Vec<&Ident> = path
            .path
            .segments
            .iter()
            .map(|segment| &segment.ident)
            .collect();
        self.found
            .extend(Self::ambient(&names).map(|()| LineNumber::from(path.span())));
        visit::visit_expr_path(self, path);
    }
}
