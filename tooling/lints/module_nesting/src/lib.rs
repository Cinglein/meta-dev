mod module_tree;

use module_tree::ModuleTree;

use task::Policy;
use task::Report;
use task::SourceFile;

#[derive(Debug)]
pub struct ModuleNesting;

impl ModuleNesting {
    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        let tree = ModuleTree::collect(files);
        Report::new(
            "modules nest by use: a file used only inside one sibling belongs inside it",
            tree.violations().chain(tree.crowded(policy)).collect(),
        )
    }
}
