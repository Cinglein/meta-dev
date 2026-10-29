use meta_pull_query::LiveData;
use meta_pulls::{Action, Departure, Patience, Stop, TransitionVisitor};
use meta_schema::PullRequest;
use meta_state::State;

use super::HeadWatch;
use super::watched::Watched;
use crate::request::cli_failure::CliFailure;
use crate::request::merge_queue::close_out::CloseOut;
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

    pub fn taken(self, patience: Patience) -> Result<Option<Watched>, CliFailure> {
        let pull_request = self.watch.pull_request();
        let head = self.live.pull(pull_request).map(|ruled| ruled.head(self.watch.trunk()));
        head.map_or(Ok(Some(Watched::Stopped(Stop::Missing(pull_request)))), |head| head.step(patience).visit(self))
    }
}

impl TransitionVisitor for Turn<'_> {
    type Output = Result<Option<Watched>, CliFailure>;

    fn act(self, action: Action) -> Self::Output {
        Ok(self.watch.acted(action))
    }

    fn conflict(self, pull_request: PullRequest) -> Self::Output {
        let shown = Listed::new(format_args!("{pull_request} conflicts with main: needs conflict fix\n"));
        Ok(Some(Watched::Flagged(shown, pull_request)))
    }

    fn stop(self, stop: Stop) -> Self::Output {
        Ok(Some(Watched::Stopped(stop)))
    }

    fn left(self, departure: Departure) -> Self::Output {
        let shown = Listed::new(format_args!("{departure}\n"));
        let closing = |landed| {
            let close_out = CloseOut::judged(&self.state, &self.live, landed);
            self.watch.closed(close_out).map_or_else(
                |failure| Watched::Flagged(shown.clone().then(format_args!("close-out failed: {failure}\n")), landed),
                |closed| Watched::Left(shown.clone().then(closed)),
            )
        };
        Ok(Some(departure.landed().map_or_else(|| Watched::Left(shown.clone()), closing)))
    }
}
