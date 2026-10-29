use super::held::Held;
use crate::request::local::local_clone::vetting::cause::Cause;

#[derive(Debug)]
pub struct Unsupported;

impl Unsupported {
    const UNKNOWN: &'static str = "no process table on this platform";

    pub fn held() -> Result<Held, Cause> {
        Err(Cause::new(Self::UNKNOWN))
    }
}
