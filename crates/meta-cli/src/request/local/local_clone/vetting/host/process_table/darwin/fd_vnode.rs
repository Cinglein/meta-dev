use std::mem::size_of;
use std::path::PathBuf;

use libproc::file_info::{ListFDs, PIDFDInfo, PIDFDInfoFlavor, ProcFDType, pidfdinfo};
use libproc::proc_pid::listpidinfo;

use super::probe_failure::ProbeFailure;
use super::vnode_path::VnodePath;

const FILE_INFO_SIZE: usize = size_of::<[u32; 2]>() + size_of::<libc::off_t>() + size_of::<[u32; 2]>();

#[repr(C)]
#[derive(Debug, Default)]
pub struct FdVnode {
    file: [u8; FILE_INFO_SIZE],
    vnode: VnodePath,
}

impl FdVnode {
    pub fn read(pid: i32, fd: i32, table_len: usize) -> Result<Option<PathBuf>, ProbeFailure> {
        match pidfdinfo::<Self>(pid, fd) {
            Ok(described) => Ok(described.vnode.path()),
            Err(failure) => {
                let open = listpidinfo::<ListFDs>(pid, table_len)?;
                let vnodes = open.iter().filter(|open| matches!(ProcFDType::from(open.proc_fdtype), ProcFDType::VNode));
                vnodes
                    .map(|open| open.proc_fd)
                    .all(|open| open != fd)
                    .then_some(None)
                    .ok_or_else(|| ProbeFailure::from(failure))
            }
        }
    }
}

impl PIDFDInfo for FdVnode {
    fn flavor() -> PIDFDInfoFlavor {
        PIDFDInfoFlavor::VNodePathInfo
    }
}
