use std::ffi::OsString;
use std::path::{Path, PathBuf};

use meta_run::{Budget, Port, Run};

use super::cause::Cause;
use super::change::Change;
use super::kept::Kept;
use super::query::Query;
use super::vet_argv::VetArgv;

#[derive(Clone, Debug)]
pub struct Repo {
    git_dir: PathBuf,
    work_tree: Option<PathBuf>,
    budget: Budget,
}

impl Repo {
    const ISOLATED: [(&'static str, &'static str); 5] = [
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_SYSTEM", "/dev/null"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_NO_REPLACE_OBJECTS", "1"),
        ("GIT_CONFIG_COUNT", "7"),
    ];
    const PINNED: [(&'static str, &'static str); 7] = [
        ("core.trustctime", "true"),
        ("core.checkStat", "default"),
        ("core.fsmonitor", "false"),
        ("core.untrackedCache", "false"),
        ("core.ignoreStat", "false"),
        ("core.sparseCheckout", "false"),
        ("status.showUntrackedFiles", "all"),
    ];

    pub fn common(git_dir: &Path, budget: Budget) -> Self {
        Self { git_dir: git_dir.to_path_buf(), work_tree: None, budget }
    }

    pub fn worktree(git_dir: &Path, work_tree: &Path, budget: Budget) -> Self {
        Self { git_dir: git_dir.to_path_buf(), work_tree: Some(work_tree.to_path_buf()), budget }
    }

    pub fn observe<Asked: Query>(&self, query: &Asked) -> Result<Asked::View, Kept> {
        let run = self.run(Asked::ARGV, query.operands());
        let run = query.input().into_iter().fold(run, Run::fed);
        query.viewed(&self.ran(run)?)
    }

    pub fn perform<Made: Change>(&self, change: &Made) -> Result<(), Kept> {
        self.ran(self.run(change.argv(), change.operands())).map(drop)
    }

    pub fn run(&self, argv: VetArgv, operands: Vec<OsString>) -> Run {
        let dir = self.work_tree.as_deref().unwrap_or(&self.git_dir);
        let run = Port::<VetArgv>::new(dir).run(argv, operands);
        let run = Self::ISOLATED.iter().fold(run, |run, (name, value)| run.var(name, value));
        let pinned = Self::PINNED.iter().enumerate().fold(run, |run, (at, (key, value))| {
            run.var(format!("GIT_CONFIG_KEY_{at}"), key).var(format!("GIT_CONFIG_VALUE_{at}"), value)
        });
        let located = pinned.var("GIT_DIR", &self.git_dir);
        self.work_tree.iter().fold(located, |run, tree| run.var("GIT_WORK_TREE", tree))
    }

    fn ran(&self, run: Run) -> Result<String, Kept> {
        let ran = run.ran(self.budget.deadline()).map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        let output = ran.stdout().text().into_owned();
        ran.success().then_some(output).ok_or_else(|| Kept::Uninspected(Cause::new(ran.stderr())))
    }
}
