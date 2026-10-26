use std::fmt;

use corpus_check::LintName;
use meta_xtask::Task;

use crate::test_fn::TestFn;
use crate::type_name::TypeName;

#[derive(Clone, Copy, Debug)]
pub enum Enforcement {
    Tracked,
    Defined,
    Lint(LintName),
    Test(TestFn),
    CiStep(Task),
    Type(TypeName),
}

impl Enforcement {
    #[must_use]
    pub fn test(&self) -> Option<&TestFn> {
        match self {
            Enforcement::Test(test) => Some(test),
            _ => None,
        }
    }
}

impl fmt::Display for Enforcement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Enforcement::Tracked => formatter.write_str("rung 1, no enforcer"),
            Enforcement::Defined => formatter.write_str("rung 2, no enforcer"),
            Enforcement::Lint(lint) => write!(formatter, "rung 3, lint {lint}"),
            Enforcement::Test(test) => write!(formatter, "rung 3, test {test}"),
            Enforcement::CiStep(task) => write!(formatter, "rung 3, ci step {}", <&str>::from(*task)),
            Enforcement::Type(type_name) => write!(formatter, "rung 4, type {type_name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use super::{Enforcement, LintName, Task, TestFn, TypeName};
    use crate::spec_view::SpecView;

    const THIS_FILE: &str = "self/spec/src/enforcement.rs";
    const THIS_TEST: &str = "each_rung_shows_its_enforcer_and_only_an_existing_test_enforces";
    const ABSENT: &str = "absent";
    const SHOWN: [&str; 6] = [
        "rung 1, no enforcer",
        "rung 2, no enforcer",
        "rung 3, lint no-comments",
        "rung 3, test each_rung_shows_its_enforcer_and_only_an_existing_test_enforces in self/spec/src/enforcement.rs",
        "rung 3, ci step lint",
        "rung 4, type meta_spec::type_name::TypeName",
    ];
    const TEST: TestFn = TestFn::new(THIS_FILE, THIS_TEST);
    const ALL: [Enforcement; 6] = [
        Enforcement::Tracked,
        Enforcement::Defined,
        Enforcement::Lint(LintName::NoComments),
        Enforcement::Test(TEST),
        Enforcement::CiStep(Task::Lint),
        Enforcement::Type(TypeName::new(type_name::<TypeName>)),
    ];

    #[test]
    fn each_rung_shows_its_enforcer_and_only_an_existing_test_enforces() {
        assert_eq!(ALL.map(|enforcement| enforcement.to_string()), SHOWN);
        assert_eq!(ALL.iter().filter_map(Enforcement::test).collect::<Vec<_>>(), [&TEST]);
        assert!(
            TEST.exists_under(&SpecView::root()) && !TestFn::new(THIS_FILE, ABSENT).exists_under(&SpecView::root())
        );
    }
}
