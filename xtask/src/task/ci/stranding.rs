use std::path::Path;

use meta_log::{GitLog, LogFailure, LogStore, RemoteName, Snapshot};
use meta_severity::Scales;
use meta_state::State;

use crate::task::failure::Failure;

pub struct Stranding;

impl Stranding {
    const REMOTE: &str = "origin";

    pub fn refuse_stranded(root: &Path, scales: &Scales) -> Result<(), Failure> {
        let remote: RemoteName = Self::REMOTE.parse().unwrap_or_else(|never| match never {});
        Self::judged(scales, GitLog::new(root.to_path_buf(), remote).read())
    }

    fn judged(scales: &Scales, read: Result<Snapshot, LogFailure>) -> Result<(), Failure> {
        read.map_or_else(
            |unread| {
                println!("severity: meta/log unread, stranded items not checked: {unread}");
                Ok(())
            },
            |snapshot| {
                let strands = scales.stranded(&State::fold(snapshot.records()));
                (!strands.items().is_empty())
                    .then_some(strands)
                    .map_or(Ok(()), |strands| Err(Failure::Stranded(strands)))
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use meta_log::{LogFailure, LogStore, MemoryLog};
    use meta_schema::{Event, NewConcern, Record};
    use meta_severity::Scales;

    use super::Stranding;

    const TOP: &str = "features-and-maintenance";
    const MAJOR: &str = "major";
    const OVERRIDDEN: &str =
        r#"{"features-and-maintenance": [{"name": "blocker", "meaning": "x"}, {"name": "nuisance", "meaning": "x"}]}"#;

    #[test]
    fn a_policy_that_strands_a_rated_item_fails_and_an_unread_log_is_skipped() {
        let log = MemoryLog::default();
        let concern =
            NewConcern::new(TOP.parse().unwrap(), TOP.parse().unwrap(), None, vec![]).rated(MAJOR.parse().unwrap());
        let record = Record::new(TOP.parse().unwrap(), Event::ConcernIdentified(concern));
        log.append::<LogFailure>(&mut |_| Ok(record.clone())).unwrap();
        let overridden: Scales = serde_json::from_str(OVERRIDDEN).unwrap();
        assert!(Stranding::judged(&overridden, log.read()).unwrap_err().to_string().contains(MAJOR));
        assert!(Stranding::judged(&Scales::default(), log.read()).is_ok());
        assert!(Stranding::judged(&overridden, Err(LogFailure::from(std::io::Error::other(TOP)))).is_ok());
    }
}
