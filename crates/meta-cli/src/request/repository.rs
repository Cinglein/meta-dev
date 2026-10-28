use std::ffi::OsString;
use std::fmt;
use std::str::FromStr;

use crate::request::ci::Commit;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository(String);

impl Repository {
    const SUFFIX: &'static str = ".git";
    const SEPARATORS: [char; 2] = ['/', ':'];
    const FIELD: &'static str = "-f";

    pub fn variables(&self, commit: &Commit) -> impl Iterator<Item = OsString> {
        let (owner, name) = self.0.split_once('/').unwrap_or_default();
        let values = [format!("sha={commit}"), format!("owner={owner}"), format!("name={name}")];
        values.into_iter().flat_map(|value| [OsString::from(Self::FIELD), OsString::from(value)])
    }
}

impl fmt::Display for Repository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for Repository {
    type Err = ();

    fn from_str(url: &str) -> Result<Self, ()> {
        let trimmed = url.trim().trim_end_matches('/');
        let mut parts = trimmed.strip_suffix(Self::SUFFIX).unwrap_or(trimmed).rsplit(Self::SEPARATORS);
        let [name, owner] = [parts.next(), parts.next()].map(|part| part.filter(|part| !part.is_empty()));
        Ok(Self(format!("{}/{}", owner.ok_or(())?, name.ok_or(())?)))
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::Repository;

    const URLS: [&str; 3] = ["https://github.com/o/r.git", "git@github.com:o/r", "ssh://git@github.com/o/r/"];
    const VARIABLES: [&str; 6] = ["-f", "sha=c", "-f", "owner=o", "-f", "name=r"];
    const COMMIT: &str = "c";
    const BARE: &str = "r";

    #[test]
    fn https_ssh_and_scp_urls_name_the_owner_and_repository() {
        let Ok(commit) = COMMIT.parse();
        let variables = URLS.map(|url| url.parse::<Repository>().unwrap().variables(&commit).collect::<Vec<_>>());
        let expected = VARIABLES.map(OsString::from);
        assert!(variables.iter().all(|variables| *variables == expected) && BARE.parse::<Repository>().is_err());
    }
}
