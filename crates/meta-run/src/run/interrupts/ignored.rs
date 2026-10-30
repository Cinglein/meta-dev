use std::ffi::c_int;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ignored(u64);

impl Ignored {
    const RADIX: u32 = 16;
    const BASE: u64 = 2;
    const STATUS: &'static str = "/proc/self/status";
    const PREFIX: &'static str = "SigIgn:";

    pub fn at_start() -> Self {
        let status = std::fs::read_to_string(Self::STATUS).unwrap_or_default();
        status.lines().find_map(|line| line.strip_prefix(Self::PREFIX)?.parse().ok()).unwrap_or_default()
    }

    pub fn spares(self, signal: c_int) -> Option<()> {
        let bit = Self::BASE.checked_pow(u32::try_from(signal - 1).ok()?)?;
        (self.0 & bit == 0).then_some(())
    }
}

impl FromStr for Ignored {
    type Err = ParseIntError;

    fn from_str(mask: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(mask.trim(), Self::RADIX).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};

    use super::Ignored;

    const NOHUP: &str = "\t0000000000000001";
    const SHELL_BACKGROUND: &str = "\t0000000000000006";
    const SIGNALS: [i32; 3] = [SIGHUP, SIGINT, SIGTERM];
    const SPARED: [[bool; 3]; 2] = [[false, true, true], [true, false, true]];

    #[test]
    fn a_status_mask_spares_exactly_the_signals_whose_bits_are_clear() {
        let spared = |mask: &str| {
            let ignored: Ignored = mask.parse().unwrap();
            SIGNALS.map(|signal| ignored.spares(signal).is_some())
        };
        assert_eq!([spared(NOHUP), spared(SHELL_BACKGROUND)], SPARED);
    }
}
