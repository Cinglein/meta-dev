use std::num::NonZeroUsize;
use std::panic::resume_unwind;
use std::thread::{Scope, ScopedJoinHandle};

use meta_pull_query::Rulebook;
use meta_pulls::{BaseBranch, RequiredChecks, RuleSource};
use meta_status::GithubFailure;

use super::LiveGithub;

#[derive(Clone, Copy, Debug)]
pub struct BaseRules<'github> {
    github: &'github LiveGithub,
}

impl<'github> BaseRules<'github> {
    pub fn new(github: &'github LiveGithub) -> Self {
        Self { github }
    }

    pub fn read(self, bases: Vec<BaseBranch>) -> Result<Rulebook, GithubFailure> {
        let width = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        let read: Vec<_> = bases.chunks(width).flat_map(|chunk| self.together(chunk)).collect();
        bases.into_iter().zip(read).map(|(base, read)| read.map(|required| (base, required))).collect()
    }

    fn together(self, chunk: &[BaseBranch]) -> Vec<Result<RequiredChecks, GithubFailure>> {
        std::thread::scope(|scope| {
            let running: Vec<_> = chunk.iter().map(|base| self.started(scope, base)).collect();
            running.into_iter().map(Self::joined).collect()
        })
    }

    fn started<'scope>(
        &'scope self,
        scope: &'scope Scope<'scope, '_>,
        base: &'scope BaseBranch,
    ) -> ScopedJoinHandle<'scope, Result<RequiredChecks, GithubFailure>> {
        scope.spawn(move || self.both(base))
    }

    fn both(self, base: &BaseBranch) -> Result<RequiredChecks, GithubFailure> {
        std::thread::scope(|scope| {
            let protection = scope.spawn(|| self.github.required(RuleSource::Protection, base));
            let rulesets = self.github.required(RuleSource::Rulesets, base)?;
            Ok(rulesets.united(Self::joined(protection)?))
        })
    }

    fn joined<Value>(run: ScopedJoinHandle<'_, Value>) -> Value {
        run.join().unwrap_or_else(|panic| resume_unwind(panic))
    }
}
