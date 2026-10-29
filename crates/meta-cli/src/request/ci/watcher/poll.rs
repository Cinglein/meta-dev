use super::gh::Observation;
use super::note::Note;
use crate::request::ci::ci_status::CiStatus;
use crate::request::output::Listed;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Poll {
    reported: Option<String>,
    appeared: Option<()>,
    fresh: Option<Note>,
}

impl Poll {
    #[must_use]
    pub fn step(self, observed: &Result<Observation, Listed>, current: Option<CiStatus>, overdue: Option<()>) -> Self {
        let appeared = self.appeared.or(observed.as_ref().ok().and_then(Observation::appeared).map(drop));
        let absent = current.filter(|status| *status == CiStatus::Absent);
        let held = current.filter(|status| *status == CiStatus::Failed).unwrap_or(CiStatus::Running);
        let event = match observed {
            Err(trouble) => Some(Note::new(trouble, held)),
            Ok(observed) => observed
                .failed_job()
                .map(Note::job_failed)
                .or(overdue.filter(|()| appeared.is_none()).map(|()| Note::awaited(None)))
                .or(observed.appeared().and(absent).map(|_| Note::awaited(Some(())))),
        };
        let fresh = event.filter(|note| self.reported.as_deref() != Some(note.line()));
        let reported = fresh.as_ref().map(|note| note.line().to_owned()).or(self.reported);
        Self { reported, appeared, fresh }
    }

    pub fn fresh(&self) -> Option<&Note> {
        self.fresh.as_ref()
    }
}

#[cfg(test)]
mod tests;
