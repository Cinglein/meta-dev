#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleSource {
    Rulesets,
    Protection,
}

impl RuleSource {
    pub const BOTH: [Self; 2] = [Self::Rulesets, Self::Protection];

    #[must_use]
    pub fn filter(self) -> &'static str {
        match self {
            Self::Rulesets => {
                "[.[] | select(.type == \"required_status_checks\") | .parameters.required_status_checks[].context]"
            }
            Self::Protection => "[.protection.required_status_checks.checks[]?.context]",
        }
    }
}
