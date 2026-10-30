#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Force {
    Hard,
    Lease,
}

impl Force {
    const PLUS: char = '+';
    const SHORT: char = '-';
    const SHORT_FORCE: char = 'f';
    const LONG: &'static str = "--";
    const VALUE: char = '=';
    const FORCE: &'static str = "--force";
    const LEASES: &'static [&'static str] = &["--force-with-lease", "--force-if-includes"];
    const SHORTEST: &'static str = "--for";

    pub fn in_word(word: &str) -> Option<Self> {
        let name = word.split(Self::VALUE).next().unwrap_or(word);
        match word {
            word if word.starts_with(Self::PLUS) => Some(Self::Hard),
            word if word.starts_with(Self::LONG)
                && name.starts_with(Self::SHORTEST)
                && [Self::FORCE].iter().chain(Self::LEASES).any(|full| full.starts_with(name)) =>
            {
                Some(if name.len() > Self::FORCE.len() { Self::Lease } else { Self::Hard })
            }
            word if word.starts_with(Self::LONG) || !word.starts_with(Self::SHORT) => None,
            word => word.contains(Self::SHORT_FORCE).then_some(Self::Hard),
        }
    }
}
