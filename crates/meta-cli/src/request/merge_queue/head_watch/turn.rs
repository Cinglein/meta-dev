use meta_pull_query::LiveData;
use meta_pulls::{Action, Departure, Detour, Patience, Stop, TransitionVisitor};
use meta_state::State;

use super::HeadWatch;
use crate::request::cli_failure::CliFailure;
use crate::request::merge_queue::close_out::CloseOut;
use crate::request::merge_queue::watched::Watched;
use crate::request::output::Listed;

#[derive(Debug)]
pub struct Turn<'watch> {
    watch: &'watch HeadWatch<'watch>,
    state: State,
    live: LiveData,
}

impl<'watch> Turn<'watch> {
    pub fn new(watch: &'watch HeadWatch<'watch>, state: State, live: LiveData) -> Self {
        Self { watch, state, live }
    }

    pub fn taken(self, patience: Patience) -> Option<Result<Option<Watched>, CliFailure>> {
        let transition = self.live.pull(self.watch.pull_request())?.head().step(patience);
        Some(transition.visit(self))
    }
}

impl TransitionVisitor for Turn<'_> {
    type Output = Result<Option<Watched>, CliFailure>;

    fn act(self, action: Action) -> Self::Output {
        self.watch.acted(&action);
        Ok(self.watch.paced())
    }

    fn detour(self, detour: Detour) -> Self::Output {
        Ok(Some(Watched::Detoured(detour)))
    }

    fn stop(self, stop: Stop) -> Self::Output {
        Ok(Some(Watched::Stopped(stop)))
    }

    fn left(self, departure: Departure) -> Self::Output {
        let close_out = departure.landed().map(|landed| CloseOut::judged(&self.state, &self.live, landed));
        let closed = close_out.map_or_else(|| Ok(Listed::new("")), |close_out| self.watch.closed(close_out))?;
        Ok(Some(Watched::Left(Listed::new(format_args!("{departure}\n")).then(closed))))
    }
}
