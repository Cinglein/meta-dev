use std::path::{Path, PathBuf};

use meta_git::{Query, Reads as _, Repository, Workdir, WorktreeFiles};

use super::GitPaths;
use crate::task::failure::Failure;

#[derive(Debug)]
pub struct GitReads(PathBuf);

impl GitReads {
    pub fn new(root: &Path) -> GitReads {
        GitReads(root.to_path_buf())
    }

    pub fn read<Asked: Query>(&self, query: &Asked) -> Result<Asked::View, Failure> {
        Ok(self.opened()?.read(query)?)
    }

    pub fn files(&self, listed: WorktreeFiles) -> Result<GitPaths, Failure> {
        let below = self.below()?;
        let paths = self.read(&listed)?.into_iter();
        Ok(paths.filter_map(|path| path.strip_prefix(&below).ok().map(Path::to_path_buf)).collect())
    }

    pub fn below(&self) -> Result<PathBuf, Failure> {
        let workdir = self.read(&Workdir)?.and_then(|workdir| workdir.canonicalize().ok());
        let root = self.0.canonicalize().ok();
        let below =
            workdir.zip(root).and_then(|(workdir, root)| root.strip_prefix(workdir).ok().map(Path::to_path_buf));
        below.ok_or_else(|| Failure::OutsideWorkdir(self.0.clone()))
    }

    fn opened(&self) -> Result<Repository, Failure> {
        Repository::discovered(&self.0)?.ok_or_else(|| Failure::NoRepository(self.0.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use meta_golden::{GoldenFailure, Goldens};
    use meta_run::GitArgv;

    use super::GitReads;

    impl GitReads {
        pub fn all_viewed(calls: &[GitArgv], view: fn(GitArgv, &str) -> Box<dyn Debug>) -> Result<bool, GoldenFailure> {
            let recordings = Goldens::workspace().recordings()?;
            let viewed: Vec<Vec<_>> = calls
                .iter()
                .map(|call| {
                    let recorded = recordings.iter().filter(|recording| recording.argv().calls(*call));
                    recorded
                        .map(|recording| recording.viewed(&recording.reported().map(|out| view(*call, out))))
                        .collect()
                })
                .collect();
            let every = viewed.iter().all(|views| !views.is_empty());
            viewed.into_iter().flatten().collect::<Result<(), _>>().map(|()| every)
        }
    }
}
