use std::ffi::OsString;

use meta_run::{Ran, Stderr};

use crate::remote_name::RemoteName;

pub trait RemoteChange {
    const HELPER: &'static str;

    fn words(&self, remote: &RemoteName) -> Vec<OsString>;

    fn moved(&self, here: &git2::Repository, there: &git2::Repository) -> Result<(), git2::Error>;

    fn sent(&self, ran: Ran) -> Result<(), Stderr> {
        if ran.success() { Ok(()) } else { Err(ran.stderr().clone()) }
    }
}
