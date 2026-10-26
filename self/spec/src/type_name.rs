use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct TypeName(fn() -> &'static str);

impl TypeName {
    #[must_use]
    pub const fn new(named: fn() -> &'static str) -> TypeName {
        TypeName(named)
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str((self.0)())
    }
}
