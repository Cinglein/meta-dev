use crate::task::ci::local_gate::Toolchain;
use crate::task::surfaces::Changed;

#[derive(Debug)]
pub struct Findings {
    surfaces: Changed,
    toolchain: Toolchain,
}

impl Findings {
    pub fn new(surfaces: Changed, toolchain: Toolchain) -> Findings {
        Findings { surfaces, toolchain }
    }

    pub fn surfaces(&self) -> &Changed {
        &self.surfaces
    }

    pub fn toolchain(&self) -> &Toolchain {
        &self.toolchain
    }
}
