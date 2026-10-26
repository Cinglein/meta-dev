mod callee;
mod finding;
mod forwarder;
mod forwarders;
mod workspace_types;

use finding::Finding;
use forwarder::Forwarder;
use forwarders::Forwarders;
use syn::visit::Visit;
use workspace_types::WorkspaceTypes;

use task::LineNumber;
use task::Name;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;

#[derive(Debug)]
pub struct NoForwarders;

impl NoForwarders {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        let types = WorkspaceTypes::collect(files);
        Report::new(
            "methods do their own work, they do not forward to another type of ours",
            files
                .iter()
                .flat_map(|file| {
                    Self::forwarders(file, &types)
                        .into_iter()
                        .map(move |forwarder| {
                            Violation::new(
                                Site::Line(
                                    file.path().to_owned(),
                                    LineNumber::from(forwarder.name().span()),
                                ),
                                Finding::Forwards {
                                    name: Name::from(forwarder.name()),
                                    callee: forwarder.callee().clone(),
                                },
                            )
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
    use super::NoForwarders;
    use task::SourceFile;
    const SOURCE: &str = "
struct Own; enum Mode { Fast } trait Rule { fn apply(x: u8) -> u8; }
struct W { inner: Own, raw: u64 }
impl W {
    fn to_own(&self) -> u8 { self.inner.value() } fn to_foreign(&self) -> u32 { self.raw.count_ones() }
    fn to_self(self, other: W) -> W { self.merge(other) } fn to_path(bits: u64) -> W { W::assemble(bits) }
    fn to_self_path(bits: u64) -> W { Self::assemble(bits) } fn to_std(bits: u64) -> u64 { u64::from(bits) }
    fn to_param(x: &Own) -> u8 { x.value() } fn to_foreign_param(x: &u64) -> u32 { x.count_ones() }
    fn to_enum(mode: Mode) -> u8 { mode.rank() } fn to_trait(x: u8) -> u8 { Rule::apply(x) }
    fn passes_field(&self, other: W) -> u8 { self.inner.weigh(other.raw) }
    fn passes_reference(&self, other: W) -> u8 { self.inner.weigh(&other) }
    fn constructs(bits: u64) -> W { W(bits) } fn fixes_a_call() -> W { W::assemble(0) }
    fn negates(self) -> W { -self.merge(self) } fn fixes_an_argument(self) -> W { self.merge(W::EMPTY) } fn to_other_field(other: W) -> u8 { other.inner.value() }
}
struct P<H> { hand: H } impl P<Own> { fn to_generic(&self) -> u8 { self.hand.value() } }
impl From<u64> for W { fn from(bits: u64) -> W { W::to_path(bits) } }
mod inner { impl W { fn to_inner(&self) -> u8 { self.inner.value() } } }
#[cfg(test)] mod tests { impl W { fn to_test(&self) -> u8 { self.inner.value() } } }
";
    const REPORTED: [&str; 11] = [
        "fn to_own only forwards to value", "fn to_self only forwards to merge",
        "fn to_path only forwards to W::assemble", "fn to_self_path only forwards to W",
        "fn to_param only forwards to value", "fn to_enum only forwards to rank",
        "fn to_trait only forwards to Rule::apply", "fn passes_field only forwards to weigh",
        "passes_reference only forwards to weigh", "fn to_generic only forwards to value",
        "fn to_inner only forwards to value",
    ];

    #[test]
    fn flags_single_call_bodies_that_hand_their_inputs_to_one_of_our_types_outside_tests() {
        let file = SourceFile::parse("w.rs", SOURCE.to_owned()).unwrap();
        let report = NoForwarders::report(core::slice::from_ref(&file));
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
