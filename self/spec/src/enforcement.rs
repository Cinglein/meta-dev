use std::fmt;

use corpus_check::LintName;

use crate::rust_type::RustType;

#[derive(Clone, Copy, Debug)]
pub enum Enforcement {
    Tracked,
    Defined,
    Lint(LintName),
    CiStep(meta_xtask::CiStep),
    Type(RustType),
}

impl fmt::Display for Enforcement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Enforcement::Tracked => formatter.write_str("rung 1, no enforcer"),
            Enforcement::Defined => formatter.write_str("rung 2, no enforcer"),
            Enforcement::Lint(lint) => write!(formatter, "rung 3, lint {lint}"),
            Enforcement::CiStep(step) => write!(formatter, "rung 3, ci step {step}"),
            Enforcement::Type(rust_type) => write!(formatter, "rung 4, type {rust_type}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use meta_xtask::CiStep;

    use super::{Enforcement, LintName, RustType};

    const SHOWN: [&str; 5] = [
        "rung 1, no enforcer",
        "rung 2, no enforcer",
        "rung 3, lint no-comments",
        "rung 3, ci step spec",
        "rung 4, type meta_spec::rust_type::RustType",
    ];
    const ALL: [Enforcement; 5] = [
        Enforcement::Tracked,
        Enforcement::Defined,
        Enforcement::Lint(LintName::NoComments),
        Enforcement::CiStep(CiStep::Spec),
        Enforcement::Type(RustType::naming::<RustType>()),
    ];

    #[test]
    fn each_rung_shows_its_enforcer() {
        assert_eq!(ALL.map(|enforcement| enforcement.to_string()), SHOWN);
    }
}
