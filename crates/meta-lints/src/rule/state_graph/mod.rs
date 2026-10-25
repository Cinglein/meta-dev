mod declarations;
mod edge_scan;
mod field_visibility;
mod function_shape;
mod impl_context;
mod type_name;
mod variant_matches;

use declarations::Declarations;
use edge_scan::EdgeScan;
use field_visibility::FieldVisibility;
use variant_matches::VariantMatches;

use crate::rule::report::Report;
use crate::source_file::SourceFile;

pub struct StateGraph;

impl StateGraph {
    pub fn report(files: &[SourceFile]) -> Report {
        let declarations = Declarations::collect(files);
        let edges = files.iter().fold(EdgeScan::new(&declarations), EdgeScan::scan);
        println!("state graph: {} vertices, {} edges", declarations.vertex_count(), edges.edge_count());
        Report::new(
            "state graph violated",
            files
                .iter()
                .flat_map(|file| {
                    FieldVisibility::violations(file).into_iter().chain(VariantMatches::violations(file, &declarations))
                })
                .chain(edges.violations())
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::StateGraph;
    use crate::source_file::SourceFile;

    const VERTICES: &str = "
pub struct Hub;
impl State for Hub {}
pub struct Spoke;
impl State for Spoke {}
pub enum Wire { Live(u8), Dead }
pub struct Leaky { pub inner: u8 }
impl Hub {
    fn hidden(self) -> Spoke { Spoke }
    pub fn first(self) -> Spoke { Spoke }
    pub fn again(self) -> Option<Spoke> { None }
    pub fn peek(&self) -> Spoke { Spoke }
}
";
    const MATCHER: &str = "
impl Other {
    fn label(wire: Wire) -> u8 {
        match wire { Wire::Live(n) => n.max(1), Wire::Dead => 0 }
    }
}
";
    const FILES: [(&str, &str); 2] = [("hub.rs", VERTICES), ("other.rs", MATCHER)];
    const REPORTED: [&str; 5] = [
        "hub.rs:9: is a hidden edge Hub -> Spoke",
        "Hub -> Spoke has 2 edges",
        "hub.rs:12: is a view returning vertex Spoke",
        "field inner of Leaky is pub",
        "match on Wire outside its file",
    ];

    #[test]
    fn reports_hidden_duplicate_and_view_edges_public_fields_and_foreign_matches() {
        let files = FILES.map(|(path, text)| SourceFile::parse(path.to_owned(), text.to_owned()).expect("valid rust"));
        let report = StateGraph::report(&files).to_string();
        assert!(REPORTED.iter().all(|line| report.contains(line)), "{report}");
    }
}
