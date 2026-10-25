mod edge;
mod edge_count;

pub use edge::Edge;
pub use edge_count::EdgeCount;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use syn::visit::Visit;
use syn::{ImplItem, ImplItemFn, ItemImpl, Receiver};

use super::declarations::Declarations;
use super::finding::Finding;
use super::function_shape::FunctionShape;
use super::impl_context::ImplContext;
use task::Name;
use task::Site;
use task::SourceFile;
use task::TypeName;
use task::Violation;
use task::{Limit, Policy};

pub struct EdgeScan<'scan> {
    declarations: &'scan Declarations,
    path: PathBuf,
    edges: BTreeMap<Edge, BTreeMap<Name, Site>>,
    violations: Vec<Violation>,
}

impl<'scan> EdgeScan<'scan> {
    pub fn new(declarations: &'scan Declarations) -> Self {
        EdgeScan {
            declarations,
            path: PathBuf::new(),
            edges: BTreeMap::new(),
            violations: Vec::new(),
        }
    }

    pub fn scan(&mut self, file: &SourceFile) {
        file.path().clone_into(&mut self.path);
        self.visit_file(file.syntax());
    }

    pub fn edge_count(&self) -> EdgeCount {
        EdgeCount::new(self.edges.values().map(BTreeMap::len).sum())
    }

    pub fn violations(mut self, policy: &Policy) -> Vec<Violation> {
        let most = policy.limit(Limit::EdgesPerVertex);
        let duplicates =
            self.edges
                .iter()
                .filter(|(_, names)| names.len() > 1)
                .map(|(edge, names)| {
                    Violation::new(
                        Site::Workspace,
                        Finding::DuplicateEdge {
                            edge: edge.clone(),
                            sites: names.clone(),
                        },
                    )
                });
        let fan_out = self.edges.iter().fold(
            BTreeMap::<&TypeName, BTreeSet<&Name>>::new(),
            |mut fan_out, (edge, names)| {
                fan_out
                    .entry(edge.source())
                    .or_default()
                    .extend(names.keys());
                fan_out
            },
        );
        let crowded = fan_out
            .into_iter()
            .filter(|(_, names)| !most.allows(names.len()))
            .map(|(source, names)| {
                Violation::new(
                    Site::Workspace,
                    Finding::CrowdedVertex {
                        source: source.clone(),
                        edges: EdgeCount::new(names.len()),
                        most,
                    },
                )
            });
        let structural: Vec<Violation> = duplicates.chain(crowded).collect();
        self.violations.extend(structural);
        self.violations
    }

    fn check_function(&mut self, context: &ImplContext, function: &ImplItemFn) {
        let shape = FunctionShape::describe(&self.path, function, context, self.declarations);
        self.violations.extend(
            shape
                .returns()
                .holds_vertex_in_tuple()
                .then(|| Violation::new(shape.site().clone(), Finding::TupleHoldsVertex)),
        );
        let receiver = function.sig.receiver();
        let source = shape.vertex_parameters().first();
        match (receiver, shape.returns().vertex(), source) {
            (None, Some(target), Some(source)) if context.in_trait() => {
                let edge = Edge::new(source.clone(), target.clone());
                self.record_edge(context, function, &shape, edge);
            }
            (None, Some(target), Some(_)) => {
                self.violations.push(Violation::new(
                    shape.site().clone(),
                    Finding::FreeTransition {
                        target: target.clone(),
                    },
                ));
            }
            (Some(receiver), _, _) if receiver.reference.is_some() => {
                self.violations
                    .extend(Self::view_fault(context, receiver, &shape));
            }
            (Some(_), Some(target), _) => {
                let edge = context
                    .edge_source(&shape)
                    .map(|source| Edge::new(source, target.clone()));
                match edge {
                    None => self.violations.push(Violation::new(
                        shape.site().clone(),
                        Finding::EdgeFromNonVertex {
                            target: target.clone(),
                            owner: context.self_type().clone(),
                        },
                    )),
                    Some(edge) => self.record_edge(context, function, &shape, edge),
                }
            }
            _ => {}
        }
    }

    fn record_edge(
        &mut self,
        context: &ImplContext,
        function: &ImplItemFn,
        shape: &FunctionShape,
        edge: Edge,
    ) {
        if context.is_public(function) {
            self.edges
                .entry(edge)
                .or_default()
                .entry(context.edge_name(function))
                .or_insert_with(|| shape.site().clone());
        } else {
            self.violations.push(Violation::new(
                shape.site().clone(),
                Finding::HiddenEdge { edge },
            ));
        }
    }

    fn view_fault(
        context: &ImplContext,
        receiver: &Receiver,
        shape: &FunctionShape,
    ) -> Option<Violation> {
        if receiver.mutability.is_some() {
            context.kind().forbids_mutation().then(|| {
                Violation::new(
                    shape.site().clone(),
                    Finding::MutatesVertex {
                        vertex: context.self_type().clone(),
                    },
                )
            })
        } else {
            shape
                .returns()
                .vertex()
                .filter(|_| !shape.body().projects_field())
                .map(|target| {
                    Violation::new(
                        shape.site().clone(),
                        Finding::ViewReturnsVertex {
                            target: target.clone(),
                        },
                    )
                })
        }
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
