use std::collections::BTreeSet;

use crate::components::Components;
use crate::facts::Facts;
use crate::field::Field;
use crate::field_name::FieldName;
use crate::finding::Finding;
use crate::findings::Findings;
use crate::layout::Layout;
use crate::method_key::MethodKey;
use crate::method_use::MethodUse;
use crate::resolution::Resolution;
use crate::structure::Structure;
use crate::type_key::TypeKey;
use crate::usage_node::UsageNode;

#[derive(Debug)]
pub struct Cohesion;

impl Cohesion {
    pub const RULE: &'static str = "LCOM4 per type with fields, over its self methods keyed by trait and name; nodes are those methods and the fields; edges join a method to each field it reads or writes, to each self method it calls, and to the field of each accessor it calls; an accessor (body is self.f, &self.f or self.f = value) is not a node; a bare use of self joins every field; a method that touches nothing is left out; a self call that names no method or accessor of the type links nothing; on a type none of whose methods takes &mut self or writes a field and none of whose field types names Mutex, RwLock, RefCell, Cell, OnceCell, OnceLock, LazyLock, LazyCell or a std atomic at any depth (a value or settings type), a method that reaches exactly one field is a projection: it keeps its edges, and a component made only of projections is dropped";
    pub const LIMITS: &'static str = "syntactic: fields reached through helpers or locals and trait default bodies are not seen; a call is resolved by name to every method of the type with that name and, when several accessors share the name, to the field of each; on a type that also has a &mut self method, one-field reads stay nodes and can split it; a &self facade over separate resources is not seen";

    #[must_use]
    pub fn measured(facts: &Facts) -> Findings<TypeKey, Components> {
        let structure = facts.structure();
        Findings::new(facts.types().iter().filter_map(|owner| {
            Some(Self::components(structure, owner))
                .filter(Components::is_split)
                .map(|components| Finding::new(owner.clone(), components))
        }))
        .sorted_by(|first, second| first.detail().most_first(second.detail()))
    }

    #[must_use]
    pub fn components(structure: &Structure, owner: &TypeKey) -> Components {
        let layout: Vec<&Field> = structure.layout(owner).into_iter().flat_map(Layout::fields).collect();
        let fields: BTreeSet<FieldName> = layout.iter().map(|field| field.name()).cloned().collect();
        let methods: Vec<&MethodUse> = structure.methods_of(owner).filter(|_| !fields.is_empty()).collect();
        let resolution = Resolution::new(fields, &methods);
        let value = Self::is_value(&layout, &methods);
        let reaching: Vec<(&MethodUse, BTreeSet<UsageNode>)> = methods
            .iter()
            .filter(|method| method.accessed().is_none())
            .map(|method| (*method, Self::reached(method, &resolution)))
            .collect();
        let inert: BTreeSet<MethodKey> = reaching
            .iter()
            .filter(|(_, reached)| value && Self::is_projection(reached))
            .map(|(method, _)| method.key().clone())
            .collect();
        Components::joined(
            reaching.into_iter().filter_map(|(method, reached)| Self::neighbourhood(method, reached)),
            &inert,
        )
    }

    #[must_use]
    pub fn is_value(fields: &[&Field], methods: &[&MethodUse]) -> bool {
        fields.iter().all(|field| !field.has_interior_mutability()) && methods.iter().all(|method| !method.mutates())
    }

    #[must_use]
    pub fn is_projection(reached: &BTreeSet<UsageNode>) -> bool {
        matches!(reached.iter().collect::<Vec<_>>().as_slice(), [UsageNode::Field(_)])
    }

    #[must_use]
    pub fn reached(method: &MethodUse, resolution: &Resolution) -> BTreeSet<UsageNode> {
        let own = UsageNode::Own(method.key().clone());
        method
            .usage()
            .touched()
            .iter()
            .cloned()
            .flat_map(|node| node.resolved(resolution))
            .filter(|node| *node != own)
            .collect()
    }

    #[must_use]
    pub fn neighbourhood(method: &MethodUse, reached: BTreeSet<UsageNode>) -> Option<BTreeSet<UsageNode>> {
        Some(reached)
            .filter(|reached| !reached.is_empty())
            .map(|reached| reached.into_iter().chain([UsageNode::Own(method.key().clone())]).collect())
    }
}

#[cfg(test)]
mod tests {
    use settings::EmbeddedDefaults;
    use task::SourceFile;

    use crate::{EleganceSettings, Measures, SynSource};

    const HEADING: &str = "B13";
    const FIRST_FINDING_LINE: &str = "  found";
    const INDENT: &str = "  ";
    const PATH: &str = "rows/src/lib.rs";
    const CRATE: &str = "rows";

    const ROWS: [[&str; 3]; 10] = [
        [
            "value type: single-field methods are projections",
            "pub struct Hold { a: A, b: B } impl Hold { pub fn a_len(&self) -> usize { self.a.len() } pub fn b_len(&self) -> usize { self.b.len() } }",
            "  found 0",
        ],
        [
            "mutable type: single-field methods stay",
            "pub struct Cell { a: A, b: B } impl Cell { pub fn a_bump(&mut self) { self.a.push(1) } pub fn b_bump(&mut self) { self.b.push(1) } }",
            "  found 1\n    rows::Cell: 2 components [a_bump] [b_bump]",
        ],
        [
            "interior mutability makes a type no value type",
            "pub struct Gauge { a: A, b: Mutex<B> } impl Gauge { pub fn a_len(&self) -> usize { self.a.len() } pub fn b_len(&self) -> usize { self.b.lock().len() } }",
            "  found 1\n    rows::Gauge: 2 components [a_len] [b_len]",
        ],
        [
            "interior mutability is found inside Arc",
            "pub struct Gauge { a: A, b: Arc<Mutex<B>> } impl Gauge { pub fn a_len(&self) -> usize { self.a.len() } pub fn b_len(&self) -> usize { self.b.lock().len() } }",
            "  found 1\n    rows::Gauge: 2 components [a_len] [b_len]",
        ],
        [
            "interior mutability is found in an atomic",
            "pub struct Gauge { a: A, b: AtomicUsize } impl Gauge { pub fn a_len(&self) -> usize { self.a.len() } pub fn b_len(&self) -> usize { self.b.load() } }",
            "  found 1\n    rows::Gauge: 2 components [a_len] [b_len]",
        ],
        [
            "value type: two multi-field methods stay apart",
            "pub struct Hold { a: A, b: B, c: C, d: D } impl Hold { pub fn ab(&self) -> usize { self.a.len() + self.b.len() } pub fn cd(&self) -> usize { self.c.len() + self.d.len() } }",
            "  found 1\n    rows::Hold: 2 components [ab] [cd]",
        ],
        [
            "value type: a method calling a projection keeps the link",
            "pub struct Hold { a: A, c: C } impl Hold { pub fn p(&self) -> usize { self.a.len() } pub fn s(&self) -> usize { self.p() + 1 } pub fn t(&self) -> usize { self.a.len() + self.c.len() } }",
            "  found 0",
        ],
        [
            "unresolved self call does not link",
            "pub struct Stub { a: A, b: B } impl Stub { pub fn first(&mut self) { self.a = 1; self.shared() } pub fn second(&mut self) { self.b = 2; self.shared() } }",
            "  found 1\n    rows::Stub: 2 components [first] [second]",
        ],
        [
            "a bare use of self joins every field",
            "pub struct Wide { a: A, b: B } impl Wide { pub fn whole(&self) -> W { W::from(self) } pub fn a_bump(&mut self) { self.a.push(1) } pub fn b_bump(&mut self) { self.b.push(1) } }",
            "  found 0",
        ],
        [
            "duplicate accessor names reach every field",
            "pub struct Twin { a: A, b: B } pub trait Named { fn name(&self) -> &B; } impl Twin { pub fn name(&self) -> &A { &self.a } pub fn show(&self) -> usize { self.name().len() } pub fn a_bump(&mut self) { self.a.push(1) } pub fn b_bump(&mut self) { self.b.push(1) } } impl Named for Twin { fn name(&self) -> &B { &self.b } }",
            "  found 0",
        ],
    ];

    struct Rows;

    impl Rows {
        fn found(source: &str) -> String {
            let file = SourceFile::parse(PATH, source.to_owned()).unwrap();
            let files = [file.in_crate(CRATE.parse().unwrap())];
            let report = Measures::measured(&SynSource::new(&files), &EleganceSettings::defaults()).to_string();
            let section: Vec<&str> = report
                .lines()
                .skip_while(|line| !line.starts_with(HEADING))
                .skip_while(|line| !line.starts_with(FIRST_FINDING_LINE))
                .take_while(|line| line.starts_with(INDENT))
                .collect();
            section.join("\n")
        }
    }

    #[test]
    fn the_split_rows_hold() {
        let wrong: Vec<String> = ROWS
            .iter()
            .map(|[name, source, expected]| (name, expected, Rows::found(source)))
            .filter(|(_, expected, actual)| actual != *expected)
            .map(|(name, expected, actual)| format!("{name}: expected {expected:?}, measured {actual:?}"))
            .collect();
        assert!(wrong.is_empty(), "{wrong:#?}");
    }
}
