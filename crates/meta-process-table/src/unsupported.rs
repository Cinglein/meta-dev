use super::held::Held;
use super::table_failure::TableFailure;

#[derive(Debug)]
pub struct Unsupported;

impl Unsupported {
    const UNKNOWN: &'static str = "no process table on this platform";

    pub fn held() -> Result<Held, TableFailure> {
        Err(TableFailure::new(Self::UNKNOWN))
    }
}
