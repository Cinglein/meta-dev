use std::path::Path;

use crate::task::failure::Failure;

pub struct TrackedBuildOutput;

impl TrackedBuildOutput {
    pub fn refuse(_root: &Path) -> Result<(), Failure> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::TrackedBuildOutput;
    use crate::task::fixture::Fixture;

    const TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n# created by cargo\n";
    const SOURCE: [(&str, &str); 2] = [("src/lib.rs", "x"), ("target-notes/readme.txt", "kept")];
    const CONFIGURED: [(&str, &str); 2] = [("target.noindex/release/meta", "x"), ("target.noindex/debug/y", "y")];
    const NESTED: [(&str, &str); 2] = [("crates/a/out/CACHEDIR.TAG", TAG), ("crates/a/out/debug/z", "z")];
    const LOOKALIKE: [(&str, &str); 2] = [("docs/CACHEDIR.TAG", "not cargo's\n"), ("docs/page.md", "text")];
    const REMEDY: &str = "git rm -r --cached";

    fn fixture(files: &[&[(&str, &str)]]) -> Fixture {
        let fixture = Fixture::new();
        files.iter().flat_map(|group| group.iter()).for_each(|(path, text)| {
            fixture.write(path, text);
        });
        fixture.committed();
        fixture
    }

    #[test]
    fn a_tracked_file_in_the_configured_target_dir_or_a_cargo_cachedir_tag_directory_is_refused_with_the_remedy() {
        let clean = fixture(&[&SOURCE, &LOOKALIKE]);
        assert!(TrackedBuildOutput::refuse(clean.root()).is_ok());
        let configured = fixture(&[&SOURCE, &CONFIGURED]);
        let nested = fixture(&[&SOURCE, &NESTED]);
        let refused = [&configured, &nested].map(|dirty| TrackedBuildOutput::refuse(dirty.root()));
        let shown = refused.map(|refused| refused.err().map(|failure| failure.to_string()).unwrap_or_default());
        assert!(shown[0].contains("target.noindex") && shown[0].contains(REMEDY), "{}", shown[0]);
        assert!(shown[1].contains("crates/a/out") && !shown[1].contains("src/lib.rs"), "{}", shown[1]);
    }
}
