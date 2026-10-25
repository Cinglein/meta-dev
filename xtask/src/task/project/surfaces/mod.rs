mod changed;

use meta_lints::{PathPrefix, Revision};
use serde::Deserialize;

pub use changed::Changed;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Surfaces(Vec<PathPrefix>);

impl Surfaces {
    pub fn union(&self, prior: Surfaces) -> Surfaces {
        Surfaces(self.0.iter().cloned().chain(prior.0).collect())
    }

    pub fn changed(&self, revision: &Revision) -> Result<Changed, meta_lints::Failure> {
        let paths = revision.changed_paths()?;
        Ok(Changed::new(paths.into_iter().filter(|path| self.0.iter().any(|surface| path.is_under(surface))).collect()))
    }
}
