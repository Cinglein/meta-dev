use affected::Changes;
use task::Failure;

pub struct Coverage;

impl Coverage {
    const AFFECTED_FLAG: &str = "--affected";

    pub fn from_flags(mut flags: impl Iterator<Item = String>) -> Result<Changes, Failure> {
        flags.try_fold(Changes::Everything, |_, flag| {
            if flag == Self::AFFECTED_FLAG {
                Ok(Changes::SinceMain)
            } else {
                Err(Failure::UnknownFlag { flag, known: Self::AFFECTED_FLAG })
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Coverage;
    use affected::Changes;
    use task::Failure;

    const AFFECTED: &str = "--affected";
    const UNKNOWN: &str = "--all";

    #[test]
    fn no_flag_covers_everything_the_affected_flag_the_changes_and_others_are_refused() {
        let parsed = |flags: &[&str]| Coverage::from_flags(flags.iter().map(ToString::to_string));
        assert_eq!(parsed(&[]).ok(), Some(Changes::Everything));
        assert_eq!(parsed(&[AFFECTED]).ok(), Some(Changes::SinceMain));
        assert!(matches!(parsed(&[UNKNOWN]), Err(Failure::UnknownFlag { .. })));
    }
}
