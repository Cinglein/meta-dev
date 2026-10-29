use crate::git_failure::GitFailure;
use crate::query::Query;

pub trait Reads {
    fn read<Asked: Query>(&self, query: &Asked) -> Result<Asked::View, GitFailure>;
}
