use std::ffi::OsString;

use meta_run::{Ran, Stderr};

use crate::remote_name::RemoteName;

pub trait RemoteChange {
    fn words(&self, remote: &RemoteName) -> Vec<OsString>;

    fn sent(&self, ran: Ran) -> Result<(), Stderr> {
        if ran.success() { Ok(()) } else { Err(ran.stderr().clone()) }
    }
}
