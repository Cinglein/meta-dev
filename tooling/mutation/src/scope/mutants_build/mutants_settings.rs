use std::str::FromStr;

use task::Failure;
use task::Site;

#[derive(Debug)]
pub struct MutantsSettings;

impl MutantsSettings {
    pub const FILE: &str = ".cargo/mutants.toml";
    const BUILD_NEUTRAL: [&str; 3] = ["exclude_globs", "exclude_re", "minimum_test_timeout"];
}

impl FromStr for MutantsSettings {
    type Err = Failure;

    fn from_str(settings: &str) -> Result<MutantsSettings, Failure> {
        let table = settings
            .parse::<toml::Table>()
            .map_err(|error| Failure::Policy { site: Site::File(Self::FILE.into()), error })?;
        match table.keys().find(|key| !Self::BUILD_NEUTRAL.contains(&key.as_str())) {
            Some(key) => Err(Failure::Refused(format!(
                "{}: {key} may change the build; add build flags to MutantsBuild::SHARED",
                Self::FILE
            ))),
            None => Ok(MutantsSettings),
        }
    }
}
