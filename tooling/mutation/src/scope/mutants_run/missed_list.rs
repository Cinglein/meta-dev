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
    pub fn uncaught_in(output: &Path) -> Result<Vec<MissedList>, Failure> {
        let listed = Uncaught::ALL.into_iter().map(|uncaught| MissedList::in_output(output, uncaught));
        let listed = listed.collect::<Result<Vec<MissedList>, Failure>>()?;
        Ok(listed.into_iter().filter(|list| !list.is_empty()).collect())
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
    use super::MissedList;
    const FILES: [&str; 2] = ["mutants.out/missed.txt", "mutants.out/timeout.txt"];
    const BLANK: &str = " \n\n";
    const TIMED_OUT: &str = "a.rs:1:1: delete x\n\nb.rs:2:2: replace y with 0\n";
    const SHOWN: [&str; 1] = [
        "2 mutants timed out, never judged: make the tests that cover each faster; if they are fast, the --timeout guard is too tight\n  a.rs:1:1: delete x\n  b.rs:2:2: replace y with 0",
    ];

    #[test]
    fn a_missing_list_fails_closed_and_only_written_ones_name_each_mutant_with_its_way_out() {
        let output = std::env::temp_dir().join(format!("meta-missed-{}", std::process::id()));
        let absent = MissedList::uncaught_in(&output).is_err();
        let [missed, timed_out] = FILES.map(|file| output.join(file));
        std::fs::create_dir_all(missed.parent().unwrap()).unwrap();
        let written = [&missed, &timed_out].into_iter().all(|file| std::fs::write(file, BLANK).is_ok());
        let clean = MissedList::uncaught_in(&output).unwrap().is_empty();
        std::fs::write(&timed_out, TIMED_OUT).unwrap();
        let shown: Vec<String> = MissedList::uncaught_in(&output).unwrap().iter().map(ToString::to_string).collect();
        std::fs::remove_dir_all(&output).unwrap();
        assert!(absent && written && clean);
        assert_eq!(shown, SHOWN);
    }
}
