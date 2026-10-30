use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use meta_git::WorktreeFiles;

use crate::task::failure::Failure;
use crate::task::revision::GitReads;
use crate::task::{BuildDirectory, TrackedBuildFiles};

pub struct TrackedBuildOutput;

impl TrackedBuildOutput {
    const TAG: &str = "CACHEDIR.TAG";
    const SIGNATURE: &str = "Signature: 8a477f597d28d172789f06886806bc55";

    pub fn refuse(root: &Path) -> Result<(), Failure> {
        let tracked: Vec<_> = GitReads::new(root).files(WorktreeFiles::Tracked)?.into_iter().collect();
        let directories: BTreeSet<&Path> = tracked.iter().flat_map(|file| Self::enclosing(file)).collect();
        let builds: BTreeSet<&Path> =
            directories.into_iter().filter(|directory| Self::is_build_directory(root, directory)).collect();
        let found: TrackedBuildFiles = tracked
            .iter()
            .filter_map(|file| {
                let outermost = Self::enclosing(file).filter(|directory| builds.contains(directory)).last();
                outermost.map(|directory| (directory.to_path_buf(), file.clone()))
            })
            .collect();
        found.none().then_some(()).ok_or(Failure::TrackedBuildOutput(found))
    }

    fn enclosing(file: &Path) -> impl Iterator<Item = &Path> {
        file.ancestors().skip(1).filter(|directory| !directory.as_os_str().is_empty())
    }

    fn is_build_directory(root: &Path, directory: &Path) -> bool {
        directory == Path::new(BuildDirectory::NAME)
            || fs::read_to_string(root.join(directory).join(Self::TAG))
                .is_ok_and(|text| text.starts_with(Self::SIGNATURE))
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
    const SHOWN: [&str; 2] = ["target.noindex", "crates/a/out"];
    const KEPT: &str = "src/lib.rs";

    impl TrackedBuildOutput {
        fn committed(files: &[&[(&str, &str)]]) -> Fixture {
            let fixture = Fixture::new();
            files.iter().flat_map(|group| group.iter()).for_each(|(path, text)| {
                fixture.write(path, text);
            });
            fixture.committed();
            fixture
        }

        fn refused(files: &[&[(&str, &str)]]) -> Option<String> {
            Self::refuse(Self::committed(files).root()).err().map(|failure| failure.to_string())
        }
    }

    #[test]
    fn a_tracked_file_in_the_configured_target_dir_or_a_cargo_cachedir_tag_directory_is_refused_with_the_remedy() {
        let clean = TrackedBuildOutput::refused(&[&SOURCE, &LOOKALIKE]);
        let [configured, nested] = [&CONFIGURED, &NESTED].map(|dirty| TrackedBuildOutput::refused(&[&SOURCE, dirty]));
        let [configured, nested] = [configured, nested].map(Option::unwrap_or_default);
        assert!(clean.is_none() && configured.contains(SHOWN[0]) && configured.contains(REMEDY));
        assert!(nested.contains(SHOWN[1]) && !nested.contains(KEPT) && !configured.contains(SHOWN[1]));
    }
}
