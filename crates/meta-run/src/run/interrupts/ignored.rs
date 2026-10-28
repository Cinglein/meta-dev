use std::ffi::c_int;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ignored(u64);

impl Ignored {
    const RADIX: u32 = 16;
    const STATUS: &'static str = "/proc/self/status";
    const PREFIX: &'static str = "SigIgn:";

    pub fn at_start() -> Self {
        let status = std::fs::read_to_string(Self::STATUS).unwrap_or_default();
        status.lines().find_map(|line| line.strip_prefix(Self::PREFIX)?.parse().ok()).unwrap_or_default()
    }

    pub fn spares(self, signal: c_int) -> Option<()> {
        (self.0 >> (signal - 1) & 1 == 0).then_some(())
    }
}

impl FromStr for Ignored {
    type Err = ParseIntError;

    fn from_str(mask: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(mask.trim(), Self::RADIX).map(Self)
    }
}
