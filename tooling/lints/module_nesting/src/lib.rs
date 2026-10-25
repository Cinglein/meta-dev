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

#[cfg(test)]
mod tests {
    use super::ModuleNesting;
    use settings::EmbeddedDefaults;
    use task::{Policy, SourceFile};
    const LIB: &str =
        "mod hub; mod spoke; mod rim; pub use hub::Hub; pub use rim::{A, B, C, D, E, F, G, H};";
    const HUB: &str = "use crate::spoke::Spoke; use crate::spoke::part::Part; pub struct Hub(Spoke, Part); pub use crate::rim::{A, B, C, D, E, F, G, H};";
    const SPOKE: &str = "mod part; use crate::rim::inner::Rim; pub struct Spoke(Rim);";
    const RIM: &str = "mod inner; mod axle; pub use inner::{Rim, A, B, C, D, E, F, G, H};";
    const FILES: [(&str, &str); 7] = [
        ("crates/w/src/lib.rs", LIB),
        ("crates/w/src/hub.rs", HUB),
        ("crates/w/src/spoke.rs", SPOKE),
        ("crates/w/src/spoke/part.rs", "pub struct Part;"),
        ("crates/w/src/rim/mod.rs", RIM),
        ("crates/w/src/rim/axle.rs", "pub struct Axle;"),
        (
            "crates/w/src/rim/inner.rs", "use super::axle::Axle; pub struct Rim(Axle);",
        ),
    ];
    const REPORTED: [&str; 4] = [
        "spoke.rs: used only from crate::hub", "axle.rs: used only from crate::rim::inner",
        "spoke.rs: reaches into crate::rim::inner", "rim/mod.rs: re-exports 9 items",
    ];

    #[test]
    fn flags_modules_used_by_one_sibling_uses_past_a_mod_rs_and_crowded_reexports_outside_the_root()
    {
        let files = FILES.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        let report = ModuleNesting::report(&files, &Policy::defaults());
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
