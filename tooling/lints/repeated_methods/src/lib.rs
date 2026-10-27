use task::{Policy, Report, SourceFile};

#[derive(Debug)]
pub struct RepeatedMethods;

impl RepeatedMethods {
    #[must_use]
    pub fn report(_: &[SourceFile], _: &Policy) -> Report {
        Report::new(
            "one public method shape on many types is a missing trait; extract the trait",
            Vec::new(),
        )
    }
}
