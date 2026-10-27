#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(u128);

impl NodeId {
    const DEVICE_SHIFT: u32 = 64;

    pub fn new(device: u64, inode: u64) -> Self {
        Self((u128::from(device) << Self::DEVICE_SHIFT) | u128::from(inode))
    }
}
