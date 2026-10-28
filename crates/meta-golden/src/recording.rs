mod status;

use std::fmt::Debug;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use meta_run::Ran;

use crate::argv::Argv;
use crate::golden::Golden;
use crate::golden_failure::GoldenFailure;
use crate::goldens::Goldens;
use crate::places::Places;
use status::Status;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recording {
    stem: PathBuf,
    argv: Argv,
    out: String,
    err: String,
    status: Status,
}

impl Recording {
    const FILES: [&'static str; 4] = ["argv", "out", "err", "status"];
    const VIEW: &'static str = "view";

    pub fn all_in(dir: &Path) -> Result<Vec<Self>, GoldenFailure> {
        let [argv, ..] = Self::FILES;
        let stems = Goldens::listed(dir, argv)?;
        stems.iter().map(|stem| Self::try_from(stem.with_extension("").as_path())).collect()
    }

    #[must_use]
    pub fn ran(stem: PathBuf, ran: &Ran, places: &Places) -> Self {
        let argv = Argv::from(ran.line().words().iter().map(|word| places.hidden(word)).collect::<Vec<_>>());
        let out = places.hidden(&ran.stdout().text());
        let err = places.hidden(&ran.stderr().to_string());
        Self { stem, argv, out, err, status: Status::from(ran) }
    }

    pub fn written(&self) -> Result<(), GoldenFailure> {
        Self::files(&self.stem).into_iter().zip(self.texts()).try_for_each(|(file, text)| file.written(&text))
    }

    pub fn checked_in(&self, dir: &Path) -> Result<(), GoldenFailure> {
        let name = self.stem.file_name().map_or_else(PathBuf::new, PathBuf::from);
        Self::files(&dir.join(name)).iter().zip(self.texts()).try_for_each(|(golden, text)| golden.checked(&text))
    }

    pub fn viewed(&self, view: &impl Debug) -> Result<(), GoldenFailure> {
        Golden::new(&self.stem.with_extension(Self::VIEW)).checked(&format!("{view:#?}\n"))
    }

    #[must_use]
    pub fn argv(&self) -> &Argv {
        &self.argv
    }

    #[must_use]
    pub fn out(&self) -> &str {
        &self.out
    }

    #[must_use]
    pub fn answered(&self) -> ExitCode {
        print!("{}", self.out);
        drop(std::io::stdout().flush());
        eprint!("{}", self.err);
        self.status.exit_code()
    }

    fn files(stem: &Path) -> [Golden; Self::FILES.len()] {
        Self::FILES.map(|extension| Golden::new(&stem.with_extension(extension)))
    }

    fn texts(&self) -> [String; Self::FILES.len()] {
        [self.argv.to_string(), self.out.clone(), self.err.clone(), self.status.to_string()]
    }
}

impl TryFrom<&Path> for Recording {
    type Error = GoldenFailure;

    fn try_from(stem: &Path) -> Result<Self, Self::Error> {
        let [argv, out, err, status] = Self::files(stem).map(|file| file.text());
        let malformed = |reason: &dyn std::fmt::Display| GoldenFailure::Malformed {
            path: stem.to_path_buf(),
            reason: reason.to_string(),
        };
        let argv = argv?.parse().map_err(|failure| malformed(&failure))?;
        let status = status?.parse().map_err(|failure| malformed(&failure))?;
        Ok(Self { stem: stem.to_path_buf(), argv, out: out?, err: err?, status })
    }
}
