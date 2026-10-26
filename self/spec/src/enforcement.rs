use std::fmt;

use corpus_check::LintName;

use crate::rust_type::RustType;
use crate::test_fn::TestFn;

#[derive(Clone, Copy, Debug)]
pub enum Enforcement {
    Tracked,
    Defined,
    Lint(LintName),
    Test(TestFn),
    CiStep(meta_xtask::CiStep),
    Type(RustType),
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
            Enforcement::CiStep(step) => write!(formatter, "rung 3, ci step {step}"),
            Enforcement::Type(rust_type) => write!(formatter, "rung 4, type {rust_type}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use meta_xtask::CiStep;

    use super::{Enforcement, LintName, RustType, TestFn};

    const SHOWN: [&str; 6] = [
        "rung 1, no enforcer",
        "rung 2, no enforcer",
        "rung 3, lint no-comments",
        "rung 3, test enforcement::tests::each_rung_shows_its_enforcer in meta-spec",
        "rung 3, ci step spec",
        "rung 4, type meta_spec::rust_type::RustType",
    ];
    const TEST: TestFn = TestFn::new("meta-spec", "enforcement::tests::each_rung_shows_its_enforcer");
    const ALL: [Enforcement; 6] = [
        Enforcement::Tracked,
        Enforcement::Defined,
        Enforcement::Lint(LintName::NoComments),
        Enforcement::Test(TEST),
        Enforcement::CiStep(CiStep::Spec),
        Enforcement::Type(RustType::naming::<RustType>()),
    ];

    #[test]
    fn each_rung_shows_its_enforcer() {
        assert_eq!(ALL.map(|enforcement| enforcement.to_string()), SHOWN);
        assert_eq!(ALL.iter().filter_map(Enforcement::test).collect::<Vec<_>>(), [&TEST]);
    }
}
