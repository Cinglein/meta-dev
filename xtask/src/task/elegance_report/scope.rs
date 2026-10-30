use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Compared,
    Skipped,
}

impl Scope {
    const TOOL: &'static str = "tooling/elegance";

    #[must_use]
    pub fn judged(changed: &[PathBuf]) -> Scope {
        if changed.iter().any(|path| path.starts_with(Path::new(Self::TOOL))) {
            Scope::Compared
        } else {
            Scope::Skipped
        }
    }

    #[must_use]
    pub fn reason(self) -> &'static str {
        match self {
            Scope::Compared => "the diff touches the elegance tool",
            Scope::Skipped => "the diff leaves the elegance tool untouched",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Scope;

    const REASON: &str = "elegance tool";
    const ROWS: [(&str, Scope); 4] = [
        ("tooling/elegance/src/cohesion.rs", Scope::Compared),
        ("tooling/elegance/goldens/meta-dev.txt", Scope::Compared),
        ("tooling/elegance_other/src/lib.rs", Scope::Skipped),
        ("crates/meta-cli/src/lib.rs", Scope::Skipped),
    ];

    #[test]
    fn the_golden_is_compared_only_when_the_elegance_tool_changes() {
        let wrong: Vec<&str> = ROWS
            .iter()
            .filter(|(path, scope)| Scope::judged(&[PathBuf::from(path)]) != *scope || !scope.reason().contains(REASON))
            .map(|(path, _)| *path)
            .collect();
        assert!(wrong.is_empty(), "{wrong:?}");
    }
}
