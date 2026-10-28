use std::ffi::OsString;

use crate::golden_failure::GoldenFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Check,
    Bless,
}

impl Mode {
    pub const BLESS: &'static str = "META_GOLDENS_BLESS";
    const CI: &'static str = "CI";

    pub fn current() -> Result<Self, GoldenFailure> {
        Self::chosen(std::env::var_os(Self::BLESS), std::env::var_os(Self::CI))
    }

    fn chosen(bless: Option<OsString>, ci: Option<OsString>) -> Result<Self, GoldenFailure> {
        match (bless, ci) {
            (None, _) => Ok(Self::Check),
            (Some(_), None) => Ok(Self::Bless),
            (Some(_), Some(_)) => Err(GoldenFailure::BlessedInCi),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::Mode;

    const SET: &str = "1";

    #[test]
    fn goldens_are_checked_unless_blessed_and_never_blessed_in_ci() {
        let set = || Some(OsString::from(SET));
        let chosen = [(None, set()), (set(), None), (set(), set())].map(|(bless, ci)| Mode::chosen(bless, ci).ok());
        assert_eq!(chosen, [Some(Mode::Check), Some(Mode::Bless), None]);
    }
}
