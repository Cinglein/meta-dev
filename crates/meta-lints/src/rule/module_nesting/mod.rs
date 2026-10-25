mod module_tree;

use module_tree::ModuleTree;

use crate::rule::report::Report;
use crate::source_file::SourceFile;

pub struct ModuleNesting;

impl ModuleNesting {
    pub fn report(files: &[SourceFile]) -> Report {
        let tree = ModuleTree::collect(files);
        Report::new(
            "modules nest by use: a file used only inside one sibling belongs inside it",
            tree.misplaced().chain(tree.deep_uses()).chain(tree.crowded()).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ModuleNesting;
    use crate::source_file::SourceFile;

    const LIB: &str = "mod hub; mod spoke; mod rim; pub use hub::Hub;";
    const HUB: &str = "use crate::spoke::Spoke; pub struct Hub(Spoke);";
    const SPOKE: &str = "use crate::rim::inner::Rim; pub struct Spoke(Rim);";
    const RIM: &str = "mod inner; pub use inner::Rim;";
    const FILES: [(&str, &str); 4] = [
        ("crates/w/src/lib.rs", LIB),
        ("crates/w/src/hub.rs", HUB),
        ("crates/w/src/spoke.rs", SPOKE),
        ("crates/w/src/rim/mod.rs", RIM),
    ];
    const REPORTED: [&str; 2] =
        ["spoke.rs: used only from crate::hub; move it inside that module", "spoke.rs: reaches into crate::rim::inner"];

    #[test]
    fn flags_a_module_used_by_one_sibling_and_a_use_that_bypasses_a_reexport() {
        let files = FILES.map(|(path, text)| SourceFile::parse(path.to_owned(), text.to_owned()).expect("valid rust"));
        let report = ModuleNesting::report(&files).to_string();
        assert!(REPORTED.iter().all(|line| report.contains(line)) && !report.contains("hub.rs:"), "{report}");
    }
}
