use meta_run::Stderr;

use crate::git_failure::GitFailure;
use crate::remote_change::RemoteChange;

pub trait Sends {
    fn send<Sent: RemoteChange>(&self, change: &Sent) -> Result<Result<(), Stderr>, GitFailure>;
}
