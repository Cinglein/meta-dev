use meta_schema::Division;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainFailure {
    #[error("domain {0}: not strictly inside its parent's domain")]
    Outside(Division),
    #[error("domain {0}: planning has no code scopes")]
    PlanningCode(Division),
}
