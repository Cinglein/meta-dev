use std::mem::size_of;
use std::path::PathBuf;

use libproc::proc_pid::{PIDInfo, PidInfoFlavor};

use super::vnode_path::VnodePath;

const _: () = assert!(size_of::<ProcessDirs>() == size_of::<libc::proc_vnodepathinfo>());

#[repr(C)]
#[derive(Debug, Default)]
pub struct ProcessDirs {
    cwd: VnodePath,
    root: VnodePath,
}

impl ProcessDirs {
    pub fn paths(&self) -> impl Iterator<Item = PathBuf> {
        [&self.cwd, &self.root].into_iter().filter_map(VnodePath::path)
    }
}

impl PIDInfo for ProcessDirs {
    fn flavor() -> PidInfoFlavor {
        PidInfoFlavor::VNodePathInfo
    }
}
