use crate::git_failure::GitFailure;
use crate::remote_query::RemoteQuery;

pub trait Asks {
    fn ask<Asked: RemoteQuery>(&self, query: &Asked) -> Result<Asked::View, GitFailure>;
}
