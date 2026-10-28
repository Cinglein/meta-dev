use std::ffi::OsStr;
use std::path::Path;
use std::time::{Duration, Instant};

use meta_run::{GitArgv, Port};

use crate::task::failure::Failure;

#[derive(Debug)]
pub struct GitReads(Port<GitArgv>);

impl GitReads {
    pub const NO_OPERANDS: [&str; 0] = [];
    const PATIENCE: Duration = Duration::from_secs(30);

    pub fn new(root: &Path) -> GitReads {
        GitReads(Port::new(root))
    }

    pub fn read<Word: AsRef<OsStr>>(
        &self,
        call: GitArgv,
        operands: impl IntoIterator<Item = Word>,
    ) -> Result<String, Failure> {
        let reported = self.0.reported(call, operands, Instant::now() + Self::PATIENCE).map_err(Failure::Git)?;
        Ok(reported.text().into_owned())
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
