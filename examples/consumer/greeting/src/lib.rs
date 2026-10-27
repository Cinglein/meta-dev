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
    use std::path::Path;

    use super::Greeting;

    const MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";
    const OWN_SOURCE: &str = "src/lib.rs";

    #[test]
    fn the_greeting_says_hello_in_its_package_folder_with_the_environment_cargo_gives_it() {
        let canonical = |path: &Path| path.canonicalize().ok();
        let manifest = env::var_os(MANIFEST_DIR).and_then(|dir| canonical(Path::new(&dir)));
        let folder = env::current_dir().ok().and_then(|folder| canonical(&folder));
        assert!(folder.is_some() && folder == manifest && Path::new(OWN_SOURCE).is_file());
        assert_eq!(Greeting::text(), "hello");
    }
}
