use crate::request::cli_failure::CliFailure;
use crate::request::project::BranchName;

pub trait ClaimHands {
    fn pruned(&self) -> Result<(), CliFailure>;
    fn configured(&self) -> Result<(), CliFailure>;
    fn branched(&self, base: &BranchName) -> Result<(), CliFailure>;
    fn added(&self) -> Result<(), CliFailure>;
    fn settled(&self) -> Result<(), CliFailure>;
    fn written(&self) -> Result<String, CliFailure>;
    fn untrailed(&self) -> Result<(), CliFailure>;
    fn removed(&self);
    fn deleted(&self);
}
