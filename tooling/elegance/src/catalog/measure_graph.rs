use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeasureGraph {
    TraitGraph,
    TransitionGraph,
    Types,
    Code,
    Effects,
    Tests,
}

impl fmt::Display for MeasureGraph {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            MeasureGraph::TraitGraph => "trait graph",
            MeasureGraph::TransitionGraph => "transition graph",
            MeasureGraph::Types => "types",
            MeasureGraph::Code => "code",
            MeasureGraph::Effects => "effects",
            MeasureGraph::Tests => "tests",
        })
    }
}
