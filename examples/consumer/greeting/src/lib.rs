#[derive(Debug)]
pub struct Greeting;

impl Greeting {
    #[must_use]
    pub fn text() -> &'static str {
        "hello"
    }
}

#[cfg(test)]
mod tests {
    use super::Greeting;

    #[test]
    fn the_greeting_says_hello() {
        assert_eq!(Greeting::text(), "hello");
    }
}
