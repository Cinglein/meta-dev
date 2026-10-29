mod uncaught;

use std::fmt;
use std::fs;
use std::path::Path;

use task::Failure;
use task::Site;
use uncaught::Uncaught;

pub struct MissedList {
    uncaught: Uncaught,
    mutants: Vec<String>,
}

impl MissedList {
    pub fn none_in(output: &Path) -> Result<bool, Failure> {
        let listed = Uncaught::ALL
            .into_iter()
            .map(|uncaught| MissedList::in_output(output, uncaught))
            .collect::<Result<Vec<MissedList>, Failure>>()?;
        listed.iter().filter(|list| !list.is_empty()).for_each(|list| eprintln!("{list}"));
        Ok(listed.iter().all(MissedList::is_empty))
    }

    fn in_output(output: &Path, uncaught: Uncaught) -> Result<MissedList, Failure> {
        let path = output.join(uncaught.file());
        let listed = fs::read_to_string(&path).map_err(|error| Failure::Io { site: Site::File(path), error })?;
        Ok(MissedList::parsed(&listed, uncaught))
    }

    fn parsed(listed: &str, uncaught: Uncaught) -> MissedList {
        let mutants = listed.lines().filter(|line| !line.trim().is_empty()).map(str::to_owned).collect();
        MissedList { uncaught, mutants }
    }

    fn is_empty(&self) -> bool {
        self.mutants.is_empty()
    }
}

impl fmt::Display for MissedList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} mutants {}", self.mutants.len(), self.uncaught)?;
        self.mutants.iter().try_for_each(|mutant| write!(formatter, "\n  {mutant}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{MissedList, Uncaught};
    const BLANK: &str = " \n\n";
    const TIMED_OUT: &str = "a.rs:1:1: delete x\n\nb.rs:2:2: replace y with 0\n";
    const SHOWN: &str = "2 mutants timed out, never judged: make the tests that cover each faster; if they are fast, the --timeout guard is too tight\n  a.rs:1:1: delete x\n  b.rs:2:2: replace y with 0";

    #[test]
    fn a_missing_list_fails_closed_and_a_written_one_names_each_mutant_with_its_way_out() {
        let output = std::env::temp_dir().join(format!("meta-missed-{}", std::process::id()));
        let absent = MissedList::none_in(&output).is_err();
        let [missed, timed_out] = Uncaught::ALL.map(|uncaught| output.join(uncaught.file()));
        std::fs::create_dir_all(missed.parent().unwrap()).unwrap();
        let written = [&missed, &timed_out].into_iter().all(|file| std::fs::write(file, BLANK).is_ok());
        let clean = MissedList::none_in(&output).unwrap();
        std::fs::write(&timed_out, TIMED_OUT).unwrap();
        let unjudged = MissedList::none_in(&output).unwrap();
        let present = MissedList::parsed(TIMED_OUT, Uncaught::TimedOut);
        std::fs::remove_dir_all(&output).unwrap();
        assert!(absent && written && clean && !unjudged);
        assert_eq!(present.to_string(), SHOWN);
    }
}
