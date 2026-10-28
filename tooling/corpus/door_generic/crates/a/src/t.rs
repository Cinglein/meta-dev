pub struct T; impl core::str::FromStr for T { type Err = (); fn from_str(_: &str) -> Result<T, ()> { Ok(T) } }
