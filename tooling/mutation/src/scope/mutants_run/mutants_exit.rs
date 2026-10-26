use std::process::ExitStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MutantsExit(i32);

impl MutantsExit {
    const CLEAN: MutantsExit = MutantsExit(0);
    const TIMEOUTS_ONLY: MutantsExit = MutantsExit(3);

    pub(super) fn accepts(status: ExitStatus) -> bool {
        status.code().map(MutantsExit).is_some_and(|exit| exit == Self::CLEAN || exit == Self::TIMEOUTS_ONLY)
    }
}

#[cfg(test)]
mod tests {
    use super::MutantsExit;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;
    const SHIFT: u32 = 8;
    const CODES: [(i32, bool); 3] = [(0, true), (3, true), (2, false)];
    #[test]
    fn a_clean_or_timed_out_run_is_accepted_and_a_run_with_misses_is_not() {
        let accepted = |code: i32| MutantsExit::accepts(ExitStatus::from_raw(code << SHIFT));
        assert!(CODES.iter().all(|(code, verdict)| accepted(*code) == *verdict));
    }
}
