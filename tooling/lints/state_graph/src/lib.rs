mod declarations;
mod edge_scan;
mod field_visibility;
mod finding;
mod function_shape;
mod impl_context;
mod variant_matches;

use declarations::Declarations;
use edge_scan::EdgeScan;
use field_visibility::FieldVisibility;
use variant_matches::VariantMatches;

use task::Policy;
use task::Report;
use task::SourceFile;

#[derive(Debug)]
pub struct StateGraph;

impl StateGraph {
    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        let declarations = Declarations::collect(files);
        let edges = files
            .iter()
            .fold(EdgeScan::new(&declarations), |mut edges, file| {
                edges.scan(file);
                edges
            });
        println!(
            "state graph: {} vertices, {} edges",
            declarations.vertex_count(),
            edges.edge_count()
        );
        Report::new(
            "state graph violated",
            files
                .iter()
                .flat_map(|file| {
                    FieldVisibility::violations(file)
                        .into_iter()
                        .chain(VariantMatches::violations(file, &declarations))
                })
                .chain(edges.violations(policy))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::declarations::VertexCount;
    use super::edge_scan::EdgeCount;
    use super::{Declarations, EdgeScan};
    use task::SourceFile;
    const SOURCE: &str = "pub struct Hub; impl State for Hub {} pub struct Rim; pub type Edge = Rim; impl State for Edge {} impl Hub { pub fn first(self) -> Rim { Rim } }";
    const VERTEX_COUNT: VertexCount = VertexCount::new(2);
    const EDGE_COUNT: EdgeCount = EdgeCount::new(1);

    #[test]
    fn the_printed_summary_counts_vertices_through_aliases_and_edges_once_per_pair() {
        let file = SourceFile::parse("hub.rs", SOURCE.to_owned()).unwrap();
        let declarations = Declarations::collect(core::slice::from_ref(&file));
        let mut edges = EdgeScan::new(&declarations);
        edges.scan(&file);
        assert_eq!(declarations.vertex_count(), VERTEX_COUNT);
        assert_eq!(edges.edge_count(), EDGE_COUNT);
    }
}
