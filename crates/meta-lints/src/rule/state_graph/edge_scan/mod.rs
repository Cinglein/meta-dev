mod edge;
mod edge_limit;
mod edge_message;

use edge::Edge;
use edge_limit::EdgeLimit;
use edge_message::EdgeMessage;

use std::collections::{BTreeMap, BTreeSet};

use syn::visit::Visit;
use syn::{ImplItem, ImplItemFn, ItemImpl, Receiver};

use super::declarations::Declarations;
use super::function_shape::FunctionShape;
use super::impl_context::ImplContext;
use super::type_name::TypeName;
use crate::count::Count;
use crate::rule::name::Name;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

pub struct EdgeScan<'scan> {
    declarations: &'scan Declarations,
    path: String,
    edges: BTreeMap<Edge, BTreeMap<Name, Site>>,
    violations: Vec<Violation>,
}

impl<'scan> EdgeScan<'scan> {
    pub fn new(declarations: &'scan Declarations) -> Self {
        EdgeScan { declarations, path: String::new(), edges: BTreeMap::new(), violations: Vec::new() }
    }

    pub fn scan(mut self, file: &SourceFile) -> Self {
        file.path().clone_into(&mut self.path);
        self.visit_file(file.syntax());
        self
    }

    pub fn edge_count(&self) -> Count {
        self.edges.values().map(|names| Count::of_len(names.len())).sum()
    }

    pub fn violations(mut self) -> Vec<Violation> {
        let duplicates = self
            .edges
            .iter()
            .filter(|(_, names)| names.len() > 1)
            .map(|(edge, names)| Violation::new(Site::Workspace, EdgeMessage::Duplicated(edge, names)));
        let fan_out =
            self.edges.iter().fold(BTreeMap::<&TypeName, BTreeSet<&Name>>::new(), |mut fan_out, (edge, names)| {
                fan_out.entry(edge.source()).or_default().extend(names.keys());
                fan_out
            });
        let crowded = fan_out
            .into_iter()
            .map(|(source, names)| (source, Count::of_len(names.len())))
            .filter(|(_, count)| EdgeLimit::MAX_PER_VERTEX.exceeded_by(*count))
            .map(|(source, count)| Violation::new(Site::Workspace, EdgeMessage::Crowded(source, count)));
        let structural: Vec<Violation> = duplicates.chain(crowded).collect();
        self.violations.extend(structural);
        self.violations
    }

    fn check_function(&mut self, context: &ImplContext, function: &ImplItemFn) {
        let shape = FunctionShape::describe(&self.path, function, context, self.declarations);
        self.violations.extend(
            shape
                .holds_vertex_in_tuple()
                .then(|| Violation::new(shape.site().clone(), "returns a tuple holding a vertex; return the vertex")),
        );
        match (function.sig.receiver(), shape.returns().vertex()) {
            (None, Some(target)) if context.in_trait() => shape
                .vertex_parameters()
                .first()
                .map(|source| Edge::new(source.clone(), target.clone()))
                .into_iter()
                .for_each(|edge| self.record_edge(context, function, &shape, edge)),
            (None, Some(target)) if !shape.vertex_parameters().is_empty() => {
                self.violations.push(Violation::new(shape.site().clone(), EdgeMessage::TakesVertex(target)));
            }
            (Some(receiver), _) if receiver.reference.is_some() => {
                self.check_view(context, receiver, &shape);
            }
            (Some(_), Some(target)) => {
                match context.edge_source(&shape).map(|source| Edge::new(source, target.clone())) {
                    None => self.violations.push(Violation::new(
                        shape.site().clone(),
                        EdgeMessage::FromNonVertex(target, context.self_type()),
                    )),
                    Some(edge) => self.record_edge(context, function, &shape, edge),
                }
            }
            _ => {}
        }
    }

    fn record_edge(&mut self, context: &ImplContext, function: &ImplItemFn, shape: &FunctionShape, edge: Edge) {
        if context.is_public(function) {
            self.edges
                .entry(edge)
                .or_default()
                .entry(context.edge_name(function))
                .or_insert_with(|| shape.site().clone());
        } else {
            self.violations.push(Violation::new(shape.site().clone(), EdgeMessage::Hidden(&edge)));
        }
    }

    fn check_view(&mut self, context: &ImplContext, receiver: &Receiver, shape: &FunctionShape) {
        let found = match (receiver.mutability, shape.returns().vertex()) {
            (Some(_), _) => context
                .forbids_mutation()
                .then(|| Violation::new(shape.site().clone(), EdgeMessage::MutatesVertex(context.self_type()))),
            (None, Some(target)) => (!shape.projects_field())
                .then(|| Violation::new(shape.site().clone(), EdgeMessage::ViewsVertex(target))),
            (None, None) => None,
        };
        self.violations.extend(found);
    }
}

impl<'ast> Visit<'ast> for EdgeScan<'_> {
    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let context = ImplContext::describe(item, self.declarations);
        item.items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(function) => Some(function),
                _ => None,
            })
            .for_each(|function| self.check_function(&context, function));
        syn::visit::visit_item_impl(self, item);
    }
}
