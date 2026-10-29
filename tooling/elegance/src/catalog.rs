mod direction;
mod measure;
mod measure_graph;
mod measure_id;
mod reading;

use std::fmt;

use crate::Measures;
use crate::naming::Naming;
use crate::tally::Tally;
use direction::Direction::{Higher, Lower, TargetZero};
use measure::Measure;
use measure_graph::MeasureGraph::{Code, Effects, Tests, TraitGraph, TransitionGraph, Types};
use measure_id::MeasureId;
use reading::Reading;

#[derive(Debug)]
pub struct Catalog;

impl Catalog {
    pub const MEASURES: [Measure; 24] = [
        Measure::new(MeasureId::B1, "unnamed concepts", TraitGraph, TargetZero)
            .measured(|measures| Reading::Count(measures.concepts.counted(|naming| *naming == Naming::Missing))),
        Measure::new(MeasureId::B2, "reducible traits", TraitGraph, TargetZero)
            .measured(|measures| Reading::Count(Tally::counting(measures.reducible.found()))),
        Measure::new(MeasureId::B3, "signatures declared in more than one trait", TraitGraph, TargetZero)
            .measured(|measures| Reading::Count(Tally::counting(measures.shared.found()))),
        Measure::new(MeasureId::B4, "leverage: usable methods / written bodies", TraitGraph, Higher)
            .measured(|measures| Reading::Median(measures.leverage.median())),
        Measure::new(MeasureId::B5, "order not realised: derivable hand impls", TraitGraph, TargetZero),
        Measure::new(MeasureId::B6, "label reuse: transitions per label", TransitionGraph, Higher),
        Measure::new(MeasureId::B7, "bisimilar type pairs", TransitionGraph, TargetZero),
        Measure::new(MeasureId::B8, "domain transitions touching primitives", TransitionGraph, TargetZero)
            .by_lint("primitive_boundary (partial)"),
        Measure::new(MeasureId::B9, "hidden state fields", TransitionGraph, TargetZero)
            .measured(|measures| Reading::Count(Tally::counting(measures.hidden.found()))),
        Measure::new(MeasureId::B10, "typestate share of transitions", TransitionGraph, Higher),
        Measure::new(MeasureId::B11, "transition graph simplicity", TransitionGraph, Lower),
        Measure::new(MeasureId::B12, "unnamed field concepts", Types, TargetZero)
            .measured(|measures| Reading::Count(measures.fields.counted(|naming| *naming == Naming::Missing))),
        Measure::new(MeasureId::B13, "types with LCOM4 above 1", Types, TargetZero)
            .measured(|measures| Reading::Count(Tally::counting(measures.split.found()))),
        Measure::new(MeasureId::B14, "behavioural inherent methods", Types, Lower),
        Measure::new(MeasureId::B15, "bodies identical up to types", Code, TargetZero),
        Measure::new(MeasureId::B16, "same-job paths per external effect", Effects, Lower)
            .by_lint("git_starters (partial)"),
        Measure::new(MeasureId::B17, "parsers of the same external command", Effects, Lower),
        Measure::new(MeasureId::B18, "derivable hand impls", Code, TargetZero),
        Measure::new(MeasureId::B19, "repeated method shapes across types", Code, Lower).by_lint("repeated_methods"),
        Measure::new(MeasureId::B20, "effect share and dispersion per crate", Effects, Lower).by_lint("crate_graph"),
        Measure::new(MeasureId::B21, "test duplication", Tests, Lower),
        Measure::new(MeasureId::B22, "redundant tests", Tests, TargetZero),
        Measure::new(MeasureId::B23, "repeated meta starts per command and scene", Tests, Lower),
        Measure::new(MeasureId::B24, "test lines per caught mutant", Tests, Lower),
    ];

    pub fn listed(measures: &Measures, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "catalog: {} measures, report-only", Self::MEASURES.len())?;
        Self::MEASURES
            .iter()
            .try_for_each(|measure| writeln!(formatter, "  {measure}: {}", measure.status().read(measures)))
    }
}
