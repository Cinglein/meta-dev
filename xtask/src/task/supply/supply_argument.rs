use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[repr(u8)]
pub enum SupplyArgument {
    #[strum(serialize = "deny")]
    Deny,
    #[strum(serialize = "--config")]
    Config,
    #[strum(serialize = "fetch")]
    Fetch,
    #[strum(serialize = "db")]
    Database,
    #[strum(serialize = "check")]
    Check,
    #[strum(serialize = "--hide-inclusion-graph")]
    HideGraph,
    #[strum(serialize = "advisories")]
    Advisories,
    #[strum(serialize = "bans")]
    Bans,
    #[strum(serialize = "licenses")]
    Licenses,
    #[strum(serialize = "sources")]
    Sources,
    #[strum(serialize = "semver-checks")]
    SemverChecks,
    #[strum(serialize = "--baseline-rev")]
    BaselineRevision,
    #[strum(serialize = "git")]
    History,
    #[strum(serialize = "--redact")]
    Redact,
    #[strum(serialize = "--no-banner")]
    NoBanner,
    #[strum(serialize = "--log-level=warn")]
    Quiet,
}
