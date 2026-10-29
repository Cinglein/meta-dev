use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TrackedBuildFiles(BTreeMap<PathBuf, Vec<PathBuf>>);

impl TrackedBuildFiles {
    pub fn none(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromIterator<(PathBuf, PathBuf)> for TrackedBuildFiles {
    fn from_iter<Pairs: IntoIterator<Item = (PathBuf, PathBuf)>>(pairs: Pairs) -> TrackedBuildFiles {
        TrackedBuildFiles(pairs.into_iter().fold(BTreeMap::new(), |mut found, (directory, file)| {
            found.entry(directory).or_insert_with(Vec::new).push(file);
            found
        }))
    }
}

impl fmt::Display for TrackedBuildFiles {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "git tracks build output; untrack it and ignore it:")?;
        self.0.iter().try_for_each(|(directory, files)| {
            let shown = directory.display();
            write!(
                formatter,
                "\n  {shown}: {} tracked files; git rm -r --cached {shown}, then add it to .gitignore",
                files.len()
            )
        })
    }
}
