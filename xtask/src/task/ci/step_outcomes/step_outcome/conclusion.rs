use strum::EnumString;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub(super) enum Conclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
}
