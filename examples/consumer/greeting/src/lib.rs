#[derive(Debug)]
pub struct Greeting;

impl Greeting {
    #[must_use]
    #[doc = "```\nassert_eq!(std::thread::current().name(), Some(\"main\"));\nassert_eq!(greeting::Greeting::text(), \"hello\");\n```"]
    pub fn text() -> &'static str {
        "hello"
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    use super::Greeting;

    const MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";

    #[test]
    fn the_greeting_says_hello_in_its_package_folder_with_the_environment_cargo_gives_it() {
        let manifest = env::var_os(MANIFEST_DIR).map(PathBuf::from);
        let folder = env::current_dir().ok();
        assert!(folder.is_some() && folder == manifest);
        assert_eq!(Greeting::text(), "hello");
    }
}
