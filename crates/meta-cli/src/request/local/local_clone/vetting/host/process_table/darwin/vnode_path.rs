use std::ffi::OsStr;
use std::mem::{offset_of, size_of};
use std::os::unix::ffi::OsStrExt as _;
use std::path::PathBuf;

const INFO_SIZE: usize = offset_of!(libc::vnode_info_path, vip_path);
const NAME_SIZE: usize = size_of::<libc::vnode_info_path>() - INFO_SIZE;
const _: () = assert!(size_of::<VnodePath>() == size_of::<libc::vnode_info_path>());

#[repr(C)]
#[derive(Debug)]
pub struct VnodePath {
    info: [u8; INFO_SIZE],
    name: [u8; NAME_SIZE],
}

impl VnodePath {
    const END: u8 = 0;

    pub fn path(&self) -> Option<PathBuf> {
        let named = self.name.split(|byte| *byte == Self::END).next().filter(|named| !named.is_empty())?;
        Some(PathBuf::from(OsStr::from_bytes(named)))
    }
}

impl Default for VnodePath {
    fn default() -> Self {
        Self { info: [Self::END; INFO_SIZE], name: [Self::END; NAME_SIZE] }
    }
}
