use std::process::ExitStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MutantsExit(i32);

impl MutantsExit {
    const CLEAN: MutantsExit = MutantsExit(0);
    const JUDGED: [MutantsExit; 3] = [Self::CLEAN, MutantsExit(2), MutantsExit(3)];

    pub(super) fn judged(status: ExitStatus) -> Option<MutantsExit> {
        status.code().map(MutantsExit).filter(|exit| Self::JUDGED.contains(exit))
    }

    pub(super) fn clean(self) -> bool {
        self == Self::CLEAN
    }
}

#[cfg(test)]
mod tests {
    use super::MutantsExit;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;
    const SHIFT: u32 = 8;
    const CODES: [(i32, Option<bool>); 5] = [(0, Some(true)), (3, Some(false)), (2, Some(false)), (1, None), (4, None)];
    #[test]
    fn only_a_clean_run_is_accepted_and_only_runs_that_judged_mutants_have_lists_to_read() {
        let judged = |code: i32| MutantsExit::judged(ExitStatus::from_raw(code << SHIFT)).map(MutantsExit::clean);
        assert!(CODES.iter().all(|(code, verdict)| judged(*code) == *verdict));
    }
}
