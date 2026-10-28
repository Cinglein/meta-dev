pub struct Reader; impl Reader { pub fn read<T: core::str::FromStr>(text: &str) -> Option<T> { text.parse::<T>().ok() } pub fn port() -> Option<Port> { Reader::read::<Port>("port") } }
