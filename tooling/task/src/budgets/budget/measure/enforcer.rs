#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enforcer {
    Checker,
    Lint,
    MutationJobs,
}
