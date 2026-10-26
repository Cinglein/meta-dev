use std::any::type_name;
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct RustType(fn() -> &'static str);

impl RustType {
    #[must_use]
    pub const fn naming<Named: ?Sized>() -> RustType {
        RustType(type_name::<Named>)
    }
}

impl fmt::Display for RustType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str((self.0)())
    }
}
