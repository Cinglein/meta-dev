use std::fmt;
use std::str::FromStr;

use crate::task::ci::local_gate::Unready;
use crate::task::failure::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetaRev(String);

impl MetaRev {
    const PACKAGE: &str = "[[package]]";
    const NAME: &str = "name = \"meta-xtask\"";
    const SOURCE: &str = "source = \"";
    const REV: char = '#';
    const QUOTE: char = '"';
    const REV_LENGTH: usize = 7;
    const LOCAL: &str = "local";

    pub fn local() -> MetaRev {
        MetaRev(Self::LOCAL.to_owned())
    }
}

impl FromStr for MetaRev {
    type Err = Failure;

    fn from_str(text: &str) -> Result<MetaRev, Failure> {
        let locked = text
            .split(Self::PACKAGE)
            .find(|block| block.contains(Self::NAME))
            .and_then(|block| block.lines().find_map(|line| line.trim().strip_prefix(Self::SOURCE)))
            .and_then(|source| source.rsplit_once(Self::REV))
            .map(|(_, rev)| rev.trim_end_matches(Self::QUOTE));
        let rev = locked.unwrap_or(text).trim();
        (!rev.is_empty() && rev.chars().all(|digit| digit.is_ascii_hexdigit()))
            .then(|| MetaRev(rev.chars().take(Self::REV_LENGTH).collect()))
            .ok_or(Failure::Unready(Unready::Unversioned))
    }
}

impl fmt::Display for MetaRev {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::MetaRev;

    const LOCKED: &str = "[[package]]\nname = \"meta-xtask\"\nversion = \"0.1.0\"\nsource = \"git+https://example.invalid/meta-dev?rev=0a1b2c3d4#0a1b2c3d4e5f\"\n\n[[package]]\nname = \"xtask\"\n";
    const MEMBER: &str = "[[package]]\nname = \"meta-xtask\"\nversion = \"0.1.0\"\n";
    const HEAD: &str = "9f8e7d6\n";
    const SHORT_LOCKED: &str = "0a1b2c3";

    #[test]
    fn a_git_dependency_gives_its_locked_rev_a_bare_rev_itself_and_a_member_none() {
        assert_eq!(LOCKED.parse::<MetaRev>().map(|rev| rev.to_string()).ok().as_deref(), Some(SHORT_LOCKED));
        assert_eq!(HEAD.parse::<MetaRev>().map(|rev| rev.to_string()).ok().as_deref(), Some(HEAD.trim()));
        assert!(MEMBER.parse::<MetaRev>().is_err());
    }
}
