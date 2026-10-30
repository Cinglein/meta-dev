use super::super::git_call::GitCall;
use super::super::tokens::Segment;

pub struct Redirect;

impl Redirect {
    const CHANGERS: [(&'static str, [&'static str; 2]); 2] = [("env", ["-C", "--chdir"]), ("sudo", ["-D", "--chdir"])];
    const GIT_FLAGS: [&'static str; 2] = ["--git-dir", "--work-tree"];
    const INHERITED: [&'static str; 3] = ["GIT_DIR", "GIT_WORK_TREE", "GH_REPO"];
    const VALUE: &'static str = "=";
    const DIRECTORY_CHANGERS: &'static [&'static str] = &["cd", "pushd", "popd"];
    const FOLLOWED: &'static str = "cd";
    const HOMES: &'static [&'static str] = &["$HOME", "$PWD"];
    const UNFOLLOWABLE: &'static [char] = &['$', '*', '?', '[', '`', '{', '\\', '!'];

    pub fn inherited(words: &[String]) -> Option<()> {
        words.iter().any(|word| Self::INHERITED.iter().any(|name| word.contains(name))).then_some(())
    }

    pub fn local(part: &Segment) -> Option<()> {
        let words = part.words();
        let call = GitCall::parse(words);
        let directories = call.as_ref().map(GitCall::directory_words).unwrap_or_default();
        let moved = Self::CHANGERS.iter().any(|(program, flags)| Self::changes(words, program, flags));
        let flagged = call.is_some() && words.iter().any(|word| Self::names_git_flag(word));
        let unreadable = directories.iter().any(|word| Self::followable(word).is_none());
        (moved || flagged || unreadable || (!directories.is_empty() && part.expansion().is_some())).then_some(())
    }

    pub fn followable(target: &str) -> Option<()> {
        let bare = Self::HOMES.iter().find_map(|home| target.strip_prefix(home)).unwrap_or(target);
        (!target.starts_with('-') && !bare.contains(Self::UNFOLLOWABLE)).then_some(())
    }

    pub fn unfollowable(part: &Segment) -> Option<()> {
        let words = part.words();
        let changes = words.iter().any(|word| Self::DIRECTORY_CHANGERS.contains(&word.as_str()));
        (changes && (part.expansion().is_some() || Self::plain_target(words).is_none())).then_some(())
    }

    pub fn plain_target(words: &[String]) -> Option<&str> {
        match words {
            [cd, target] if cd == Self::FOLLOWED => {
                Some(target.as_str()).filter(|target| Self::followable(target).is_some())
            }
            _ => None,
        }
    }

    fn changes(words: &[String], program: &str, flags: &[&str]) -> bool {
        let named = words.iter().any(|word| word.rsplit('/').next() == Some(program));
        named && words.iter().any(|word| flags.iter().any(|flag| word.starts_with(flag)))
    }

    fn names_git_flag(word: &str) -> bool {
        Self::GIT_FLAGS
            .iter()
            .any(|flag| word.strip_prefix(flag).is_some_and(|rest| rest.is_empty() || rest.starts_with(Self::VALUE)))
    }
}
