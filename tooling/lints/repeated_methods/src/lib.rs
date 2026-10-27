mod shape_index;

use task::{Limit, Policy, Report, SourceFile};

use shape_index::ShapeIndex;

#[derive(Debug)]
pub struct RepeatedMethods;

impl RepeatedMethods {
    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        Report::new(
            "one public method shape on many types is a missing trait; extract the trait",
            ShapeIndex::of_files(files).repeated(policy.limit(Limit::TypesPerMethodShape)),
        )
    }
}
