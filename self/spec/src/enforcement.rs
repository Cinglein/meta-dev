use std::fmt;
use std::path::Path;

use corpus_check::LintName;

use crate::enforcer::Enforcer;
use crate::rust_type::RustType;

#[derive(Clone, Copy, Debug)]
pub enum Enforcement {
    Tracked,
    Defined,
    Lint(LintName),
    CiStep(meta_xtask::CiStep),
    Check(Enforcer, &'static [Enforcer]),
    Type(RustType),
}

impl Enforcement {
    #[must_use]
    pub fn lost(&self, root: &Path) -> Option<Enforcer> {
        match self {
            Enforcement::Check(first, more) => {
                [first].into_iter().chain(*more).find(|enforcer| !enforcer.found(root)).copied()
            }
            Enforcement::Tracked
            | Enforcement::Defined
            | Enforcement::Lint(_)
            | Enforcement::CiStep(_)
            | Enforcement::Type(_) => None,
        }
    }
}

impl fmt::Display for Enforcement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Enforcement::Tracked => formatter.write_str("rung 1, no enforcer"),
            Enforcement::Defined => formatter.write_str("rung 2, no enforcer"),
            Enforcement::Lint(lint) => write!(formatter, "rung 3, lint {lint}"),
            Enforcement::CiStep(step) => write!(formatter, "rung 3, ci step {step}"),
            Enforcement::Check(first, more) => {
                write!(formatter, "rung 3, check {first}")?;
                more.iter().try_for_each(|enforcer| write!(formatter, ", check {enforcer}"))
            }
            Enforcement::Type(rust_type) => write!(formatter, "rung 4, type {rust_type}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use meta_xtask::CiStep;

    use super::{Enforcement, Enforcer, LintName, RustType};
    use crate::SpecView;

    const FILE: &str = "self/spec/src/enforcement.rs";
    const TEST: &str = "each_rung_shows_its_enforcer_and_a_check_is_found_only_by_its_name_in_its_file";
    const SHOWN: [&str; 6] = [
        "rung 1, no enforcer",
        "rung 2, no enforcer",
        "rung 3, lint no-comments",
        "rung 3, ci step spec",
        "rung 3, check self/spec/src/enforcement.rs each_rung_shows_its_enforcer_and_a_check_is_found_only_by_its_name_in_its_file, check self/spec/src/spec_view.rs check_refuses_a_stale_spec_and_any_other_argument_rewrites_it",
        "rung 4, type meta_spec::rust_type::RustType",
    ];
    const ALL: [Enforcement; 6] = [
        Enforcement::Tracked,
        Enforcement::Defined,
        Enforcement::Lint(LintName::NoComments),
        Enforcement::CiStep(CiStep::Spec),
        Enforcement::Check(
            Enforcer::new(FILE, TEST),
            &[Enforcer::new(
                "self/spec/src/spec_view.rs",
                "check_refuses_a_stale_spec_and_any_other_argument_rewrites_it",
            )],
        ),
        Enforcement::Type(RustType::naming::<RustType>()),
    ];
    const IGNORED: &str = "self/spec/src/enforcer/ignored_tests.txt";
    const MISSING: [Enforcement; 6] = [
        Enforcement::Check(Enforcer::new(FILE, "each_rung_shows_its_enforcer"), &[]),
        Enforcement::Check(Enforcer::new(FILE, "a_check_is_found_only_by_its_name_in_its_file"), &[]),
        Enforcement::Check(Enforcer::new(FILE, TEST), &[Enforcer::new(FILE, "lost")]),
        Enforcement::Check(Enforcer::new("lost.rs", TEST), &[]),
        Enforcement::Check(Enforcer::new(IGNORED, "ignored_after_test"), &[]),
        Enforcement::Check(Enforcer::new(IGNORED, "ignored_before_test"), &[]),
    ];

    #[test]
    fn each_rung_shows_its_enforcer_and_a_check_is_found_only_by_its_name_in_its_file() {
        assert_eq!(ALL.map(|enforcement| enforcement.to_string()), SHOWN);
        let root = SpecView::root();
        assert!(ALL.iter().all(|enforcement| enforcement.lost(&root).is_none()));
        assert!(MISSING.iter().all(|enforcement| enforcement.lost(&root).is_some()));
    }
}
