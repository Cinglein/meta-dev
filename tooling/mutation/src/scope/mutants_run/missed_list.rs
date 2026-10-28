use std::convert::Infallible;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use task::Failure;
use task::Site;

pub struct MissedList(Vec<String>);

impl MissedList {
    const FILE: &str = "mutants.out/missed.txt";
    pub fn in_output(output: &Path) -> Result<MissedList, Failure> {
        let path = output.join(Self::FILE);
        let listed = fs::read_to_string(&path).map_err(|error| Failure::Io { site: Site::File(path), error })?;
        let Ok(list) = listed.parse();
        Ok(list)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromStr for MissedList {
    type Err = Infallible;

    fn from_str(listed: &str) -> Result<MissedList, Infallible> {
        Ok(MissedList(listed.lines().filter(|line| !line.trim().is_empty()).map(str::to_owned).collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::MissedList;
    #[test]
    fn a_missing_list_fails_closed_and_a_written_one_counts_its_lines() {
        let output = std::env::temp_dir().join(format!("meta-missed-{}", std::process::id()));
        let absent = MissedList::in_output(&output).is_err();
        std::fs::create_dir_all(output.join(MissedList::FILE).parent().unwrap()).unwrap();
        std::fs::write(output.join(MissedList::FILE), "a.rs:1:1: delete x").unwrap();
        let present = MissedList::in_output(&output).unwrap();
        std::fs::remove_dir_all(&output).unwrap();
        assert!(absent && !present.is_empty() && " \n\n".parse::<MissedList>().is_ok_and(|blank| blank.is_empty()));
    }
}
