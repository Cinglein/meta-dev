use settings::EmbeddedDefaults;
use task::SourceFile;

use crate::{EleganceSettings, Measures, SynSource};

const HEADING: &str = "B13";
const HEADER_LINES: usize = 3;
const INDENT: &str = "  ";
const PATH: &str = "rows/src/lib.rs";
const CRATE: &str = "rows";

const ROWS: [[&str; 3]; 6] = [
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
        "value type: multi-field methods stay nodes",
        "pub struct Hold { a: A, b: B, c: C } impl Hold { pub fn both(&self) -> usize { self.a.len() + self.b.len() } pub fn c_len(&self) -> usize { self.c.len() } pub fn all(&self) -> usize { self.b.len() + self.c.len() } }",
        "  found 0",
    ],
    [
        "unresolved self call does not link",
        "pub struct Stub { a: A, b: B } impl Stub { pub fn first(&mut self) { self.a = 1; self.shared() } pub fn second(&mut self) { self.b = 2; self.shared() } }",
        "  found 1\n    rows::Stub: 2 components [first] [second]",
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
            .skip(HEADER_LINES)
            .take_while(|line| line.starts_with(INDENT))
            .collect();
        section.join("\n")
    }
}

#[test]
fn the_split_rows_hold() {
    let wrong: Vec<&str> = ROWS
        .iter()
        .filter(|[_, source, expected]| Rows::found(source) != *expected)
        .map(|[name, _, _]| *name)
        .collect();
    assert!(wrong.is_empty(), "{wrong:?}");
}
