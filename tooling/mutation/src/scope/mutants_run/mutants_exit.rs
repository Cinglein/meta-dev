use std::process::ExitStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MutantsExit(i32);

impl MutantsExit {
    const CLEAN: MutantsExit = MutantsExit(0);
    const JUDGED: [MutantsExit; 3] = [Self::CLEAN, MutantsExit(2), MutantsExit(3)];

    pub(super) fn judged(status: ExitStatus) -> bool {
        status.code().map(MutantsExit).is_some_and(|exit| Self::JUDGED.contains(&exit))
    }

    pub(super) fn accepts(status: ExitStatus) -> bool {
        status.code().map(MutantsExit) == Some(Self::CLEAN)
    }
}

#[cfg(test)]
mod tests {
    use super::MutantsExit;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;
    const SHIFT: u32 = 8;
    const CODES: [(i32, bool, bool); 5] =
        [(0, true, true), (3, false, true), (2, false, true), (1, false, false), (4, false, false)];
    #[test]
    fn only_a_clean_run_is_accepted_and_only_runs_that_judged_mutants_have_lists_to_read() {
        let status = |code: i32| ExitStatus::from_raw(code << SHIFT);
        assert!(CODES.iter().all(|(code, accepted, judged)| {
            MutantsExit::accepts(status(*code)) == *accepted && MutantsExit::judged(status(*code)) == *judged
        }));
    }
}
