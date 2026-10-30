use std::ffi::OsStr;
use std::fmt;
use std::str::FromStr;

use meta_git::ObjectId;

use crate::task::failure::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit(String);

impl Commit {
    pub fn object_id(&self) -> Result<ObjectId, Failure> {
        Ok(self.0.parse()?)
    }
}

impl From<ObjectId> for Commit {
    fn from(id: ObjectId) -> Commit {
        Commit(id.to_string())
    }
}

impl FromStr for Commit {
    type Err = Failure;

    fn from_str(text: &str) -> Result<Commit, Failure> {
        let id = text.trim();
        (!id.is_empty() && id.chars().all(|digit| digit.is_ascii_hexdigit()))
            .then(|| Commit(id.to_owned()))
            .ok_or(Failure::Task(task::Failure::NoMergeBase))
    }
}

impl AsRef<OsStr> for Commit {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Commit;

    const HEX: &str = "0a1b2c\n";
    const EMPTY: &str = "\n";
    const WORD: &str = "main";

    #[test]
    fn only_a_nonempty_hex_id_is_a_commit() {
        assert!(HEX.parse::<Commit>().is_ok());
        assert!(EMPTY.parse::<Commit>().is_err() && WORD.parse::<Commit>().is_err());
    }
}
