use std::ops::Range;

use meta_golden::Golden;
use meta_trace::Program::{Gh, Git, Meta};
use meta_trace::SpanKind::{Command, Fs, Network, Spawn};
use meta_trace::Verb::{Heartbeat, Status};
use meta_trace::{Micros, SpanId, SpanKind, SpanRecord, Trace};

use crate::request::trace_view::TraceView;

const PARENT_INDEX: [Option<usize>; 7] = [None, Some(0), Some(0), Some(0), Some(0), Some(4), Some(5)];
const KINDS: [SpanKind; 7] =
    [Command(Status), Network(Gh), Spawn(Git), Fs, Spawn(Meta), Command(Heartbeat), Spawn(Git)];
const WINDOWS: [Range<Micros>; 7] = [
    Micros::new(0)..Micros::new(10_000),
    Micros::new(1_000)..Micros::new(6_000),
    Micros::new(500)..Micros::new(1_500),
    Micros::new(6_500)..Micros::new(7_000),
    Micros::new(7_000)..Micros::new(9_500),
    Micros::new(7_000)..Micros::new(9_500),
    Micros::new(7_300)..Micros::new(8_000),
];
const CHILD_FIRST_INDEX: usize = 5;
const VIEWS: [&str; 4] = ["last", "chrome status", "last nope", "first"];

struct Fixture;

impl Fixture {
    fn at_index(index: usize) -> SpanId {
        SpanId::new(u32::from(index >= CHILD_FIRST_INDEX) + 1, index)
    }

    fn trace() -> Trace {
        let spans = KINDS.iter().zip(PARENT_INDEX).zip(WINDOWS).enumerate().map(|(index, ((kind, parent), window))| {
            let span = SpanRecord::new(Self::at_index(index), parent.map(Self::at_index), *kind, window.start);
            span.ended(window.end)
        });
        Trace::new(spans.collect())
    }
}

#[test]
fn the_view_walks_the_critical_path_by_kind_and_shows_untraced_time() {
    let trace = Fixture::trace();
    Golden::for_test(module_path!(), "view").shown(&trace).unwrap();
    let views = VIEWS.map(|view| view.parse::<TraceView>().ok());
    let chrome = TraceView::Chrome(None).shown(&trace).unwrap();
    Golden::for_test(module_path!(), "chrome")
        .debugged(&(chrome, views, views.map(|view| view.map(TraceView::verb))))
        .unwrap();
}
