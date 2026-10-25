use std::collections::BTreeMap;
use std::fmt;

use super::edge::Edge;
use super::edge_limit::EdgeLimit;
use crate::count::Count;
use crate::rule::name::Name;
use crate::rule::state_graph::type_name::TypeName;
use crate::site::Site;

#[derive(Clone, Copy, Debug)]
pub enum EdgeMessage<'scan> {
    Duplicated(&'scan Edge, &'scan BTreeMap<Name, Site>),
    Crowded(&'scan TypeName, Count),
    TakesVertex(&'scan TypeName),
    FromNonVertex(&'scan TypeName, &'scan TypeName),
    Hidden(&'scan Edge),
    MutatesVertex(&'scan TypeName),
    ViewsVertex(&'scan TypeName),
}

impl fmt::Display for EdgeMessage<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdgeMessage::Duplicated(edge, names) => {
                let sites: Vec<String> = names.iter().map(|(name, site)| format!("{name} at {site}")).collect();
                write!(formatter, "{edge} has {} edges, at most one allowed: {}", names.len(), sites.join(", "))
            }
            EdgeMessage::Crowded(source, count) => write!(
                formatter,
                "{source} has {count} edges leaving it, at most {} allowed",
                EdgeLimit::MAX_PER_VERTEX
            ),
            EdgeMessage::TakesVertex(target) => {
                write!(formatter, "takes a vertex and returns {target}; make it a method on its source")
            }
            EdgeMessage::FromNonVertex(target, source) => write!(
                formatter,
                "returns vertex {target} from non-vertex {source}; declare the vertex or move the edge"
            ),
            EdgeMessage::Hidden(edge) => write!(formatter, "is a hidden edge {edge}; edges are pub"),
            EdgeMessage::MutatesVertex(vertex) => {
                write!(formatter, "takes &mut self on vertex {vertex}; transitions consume self")
            }
            EdgeMessage::ViewsVertex(target) => {
                write!(formatter, "is a view returning vertex {target}; take self by value or return a reference")
            }
        }
    }
}
