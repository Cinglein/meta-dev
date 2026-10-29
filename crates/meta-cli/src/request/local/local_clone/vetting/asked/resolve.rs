use crate::request::local::local_clone::vetting::cause::Cause;
use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::query::Query;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;
use crate::request::local::local_clone::vetting::vetted::Resolved;

#[derive(Clone, Debug)]
pub struct Resolve(Vec<String>);

impl Resolve {
    const LINE_END: &'static str = "\n";
    const UNEXPECTED: &'static str = "unexpected git cat-file output";

    pub fn new(names: Vec<String>) -> Self {
        Self(names)
    }
}

impl Query for Resolve {
    type View = Vec<Option<String>>;
    const ARGV: VetArgv = VetArgv::Resolve;

    fn input(&self) -> Option<String> {
        Some(self.0.iter().flat_map(|name| [name.as_str(), Self::LINE_END]).collect())
    }

    fn viewed(&self, out: &str) -> Result<Vec<Option<String>>, Kept> {
        let Ok(resolved) = out.parse::<Resolved>();
        (resolved.objects().len() == self.0.len())
            .then(|| resolved.objects().to_vec())
            .ok_or_else(|| Kept::Uninspected(Cause::new(Self::UNEXPECTED)))
    }
}
