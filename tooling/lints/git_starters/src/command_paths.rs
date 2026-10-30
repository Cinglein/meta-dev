use syn::visit::{self, Visit};
use syn::{Path, UseTree};

use task::{LineNumber, SourceFile};

#[derive(Debug, Default)]
pub(crate) struct CommandPaths(Vec<LineNumber>);

impl CommandPaths {
    const MODULE: &str = "process";
    const STARTER: &str = "Command";

    pub(crate) fn found(file: &SourceFile) -> Vec<LineNumber> {
        let mut paths = CommandPaths::default();
        paths.visit_file(file.syntax());
        paths.0
    }

    fn imported(&mut self, tree: &UseTree) {
        match tree {
            UseTree::Name(name) if name.ident == Self::STARTER => {
                self.0.push(LineNumber::from(name.ident.span()));
            }
            UseTree::Rename(name) if name.ident == Self::STARTER => {
                self.0.push(LineNumber::from(name.ident.span()));
            }
            UseTree::Group(group) => group.items.iter().for_each(|item| self.imported(item)),
            UseTree::Path(_) | UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {}
        }
    }
}

impl<'ast> Visit<'ast> for CommandPaths {
    fn visit_use_tree(&mut self, tree: &'ast UseTree) {
        match tree {
            UseTree::Path(path) if path.ident == Self::MODULE => self.imported(&path.tree),
            UseTree::Path(_)
            | UseTree::Name(_)
            | UseTree::Rename(_)
            | UseTree::Glob(_)
            | UseTree::Group(_) => {
                visit::visit_use_tree(self, tree);
            }
        }
    }

    fn visit_path(&mut self, path: &'ast Path) {
        let named = path.segments.iter().zip(path.segments.iter().skip(1));
        let starter = named.filter(|(module, starter)| {
            module.ident == Self::MODULE && starter.ident == Self::STARTER
        });
        self.0
            .extend(starter.map(|(_, starter)| LineNumber::from(starter.ident.span())));
        visit::visit_path(self, path);
    }
}
