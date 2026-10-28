use std::convert::Infallible;
use std::io::ErrorKind;
use std::path::Path;
use std::str::FromStr;

use meta_door::Door;
use task::Failure;
use task::Site;

pub struct MissedList(Vec<String>);

impl MissedList {
    const FILE: &str = "mutants.out/missed.txt";
    pub fn in_output(output: &Path) -> Result<MissedList, Failure> {
        let path = output.join(Self::FILE);
        match Door::read(&path) {
            Ok(listed) => {
                let Ok(list) = listed.parse();
                Ok(list)
            }
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(MissedList(Vec::new())),
            Err(error) => Err(Failure::Io { site: Site::File(path), error }),
        }
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
    use meta_door::{Change, Door};
    #[test]
    fn a_missing_list_is_empty_and_a_written_one_counts_its_lines() {
        let scratch = tempfile::tempdir().unwrap();
        let output = scratch.path();
        let absent = MissedList::in_output(output).unwrap();
        Door::changed(output.join(MissedList::FILE), Change::DirAll).unwrap();
        let unreadable = MissedList::in_output(output).is_err();
        Door::changed(output.join(MissedList::FILE), Change::RemoveDir).unwrap();
        Door::write(output.join(MissedList::FILE), "a.rs:1:1: delete x").unwrap();
        let present = MissedList::in_output(output).unwrap();
        let Ok(blank) = " \n\n".parse::<MissedList>();
        assert!(absent.is_empty() && unreadable && !present.is_empty() && blank.is_empty());
    }
}
