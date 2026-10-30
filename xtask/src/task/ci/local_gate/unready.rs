use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum Unready {
    #[error("tree not clean")]
    Dirty,
    #[error("HEAD not pushed")]
    Unpushed,
    #[error("not exactly one pushed branch ends at HEAD")]
    Unnamed,
    #[error("gate checkout changed during the run")]
    Tampered,
    #[error("no meta-dev rev")]
    Unversioned,
    #[error("the gate did not record every step as passed")]
    Incomplete,
    #[error("rustup shows no active toolchain for the checkout")]
    NoToolchain,
}

impl Unready {
    pub fn judged<Change, Pushed>(changes: &[Change], pushed: &[Pushed]) -> Result<(), Unready> {
        match (changes.is_empty(), pushed.is_empty()) {
            (false, _) => Err(Unready::Dirty),
            (true, true) => Err(Unready::Unpushed),
            (true, false) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Unready;

    const CHANGED: &[&str] = &["consumer/src/lib.rs"];
    const PUSHED: &[&str] = &["refs/remotes/origin/main"];
    const NOTHING: &[&str] = &[];

    #[test]
    fn only_a_clean_tree_at_a_pushed_head_is_ready() {
        let judged = [(NOTHING, PUSHED), (CHANGED, PUSHED), (CHANGED, NOTHING), (NOTHING, NOTHING)]
            .map(|(changes, pushed)| Unready::judged(changes, pushed));
        assert_eq!(judged, [Ok(()), Err(Unready::Dirty), Err(Unready::Dirty), Err(Unready::Unpushed)]);
    }
}
