mod callee;
mod forwarder;
mod forwarders;
mod workspace_types;

use forwarder::Forwarder;
use forwarders::Forwarders;
use syn::visit::Visit;
use workspace_types::WorkspaceTypes;

use crate::rule::report::Report;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

pub struct NoForwarders;

impl NoForwarders {
    pub fn report(files: &[SourceFile]) -> Report {
        let types = WorkspaceTypes::collect(files);
        Report::new(
            "methods do their own work, they do not forward to another type of ours",
            files
                .iter()
                .flat_map(|file| {
                    Self::forwarders(file, &types).into_iter().map(move |forwarder| {
                        Violation::new(Site::spanned(file.path(), forwarder.name().span()), &forwarder)
                    })
                })
                .collect(),
        )
    }

    fn forwarders(file: &SourceFile, types: &WorkspaceTypes) -> Vec<Forwarder> {
        let mut scan = Forwarders::new(types);
        scan.visit_file(file.syntax());
        scan.found()
    }
}

#[cfg(test)]
mod tests {
    use super::{NoForwarders, WorkspaceTypes};
    use crate::source_file::SourceFile;

    const SOURCE: &str = "
struct Own;
struct W { inner: Own, raw: u64 }
impl W {
    fn to_own(&self) -> u8 { self.inner.value() }
    fn to_foreign(&self) -> u32 { self.raw.count_ones() }
    fn to_self(self, other: W) -> W { self.merge(other) }
    fn to_path(bits: u64) -> W { W::assemble(bits) }
    fn to_param(x: &Own) -> u8 { x.value() }
    fn constructs(bits: u64) -> W { W(bits) }
    fn negates(self) -> W { -self.merge(self) }
    fn fixes_an_argument(self) -> W { self.merge(W::EMPTY) }
}
struct P<H> { hand: H }
impl P<Own> { fn to_generic(&self) -> u8 { self.hand.value() } }
impl From<u64> for W { fn from(bits: u64) -> W { W::to_path(bits) } }
";
    const FLAGGED: [&str; 5] = ["to_own", "to_self", "to_path", "to_param", "to_generic"];

    #[test]
    fn flags_single_call_bodies_that_hand_their_inputs_to_one_of_our_types() {
        let file = SourceFile::parse(String::from("w.rs"), String::from(SOURCE)).expect("valid rust");
        let types = WorkspaceTypes::collect(std::slice::from_ref(&file));
        let names: Vec<String> =
            NoForwarders::forwarders(&file, &types).iter().map(|forwarder| forwarder.name().to_string()).collect();
        assert_eq!(names, FLAGGED);
    }
}
