use crate::addition::Addition;
use crate::file_line::FileLine;
use crate::git_failure::GitFailure;
use crate::query::Query;
use crate::remote_name::RemoteName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FirstAddition {
    pushed: RemoteName,
    needle: String,
}

impl FirstAddition {
    const REMOTES: &'static str = "refs/remotes/";
    const ADDED: char = '+';

    #[must_use]
    pub fn new(pushed: RemoteName, needle: String) -> FirstAddition {
        FirstAddition { pushed, needle }
    }

    fn in_commit(
        &self,
        repository: &git2::Repository,
        commit: &git2::Commit<'_>,
    ) -> Result<Option<Addition>, GitFailure> {
        let tree = commit.tree()?;
        let parents = commit.parents().map(|parent| parent.tree().map(Some)).collect::<Result<Vec<_>, _>>()?;
        let parents = if parents.is_empty() { vec![None] } else { parents };
        let short = commit.as_object().short_id()?;
        let short = String::from_utf8_lossy(&short).into_owned();
        let added = parents
            .iter()
            .map(|parent| self.added(repository, parent.as_ref(), &tree, &short))
            .collect::<Result<Vec<_>, GitFailure>>()?;
        Ok(added.split_first().and_then(|(first, others)| Self::everywhere(first, others)))
    }

    fn everywhere(first: &[Addition], others: &[Vec<Addition>]) -> Option<Addition> {
        first.iter().find(|line| others.iter().all(|other| other.contains(line))).cloned()
    }

    fn added(
        &self,
        repository: &git2::Repository,
        parent: Option<&git2::Tree<'_>>,
        tree: &git2::Tree<'_>,
        commit: &str,
    ) -> Result<Vec<Addition>, GitFailure> {
        let mut options = git2::DiffOptions::new();
        options.context_lines(0).force_text(true);
        let mut diff = repository.diff_tree_to_tree(parent, Some(tree), Some(&mut options))?;
        diff.find_similar(Some(git2::DiffFindOptions::new().renames(true)))?;
        let mut lines = Vec::new();
        diff.print(git2::DiffFormat::Patch, |delta, _, line| {
            let file = delta.new_file().path().map(|path| path.to_string_lossy().into_owned());
            let at = line.new_lineno().map(FileLine::from);
            let wanted = |_: &(String, FileLine)| {
                line.origin() == Self::ADDED && String::from_utf8_lossy(line.content()).contains(&self.needle)
            };
            lines.extend(file.zip(at).filter(wanted).map(|(file, at)| Addition::new(file, at, commit.to_owned())));
            true
        })?;
        Ok(lines)
    }
}

impl Query for FirstAddition {
    type View = Option<Addition>;

    fn answered(&self, repository: &git2::Repository) -> Result<Option<Addition>, GitFailure> {
        let mut walk = repository.revwalk()?;
        walk.set_sorting(git2::Sort::TIME)?;
        walk.push_head()?;
        walk.hide_glob(&[Self::REMOTES, &self.pushed.as_ref().to_string_lossy()].concat())?;
        walk.map(|oid| self.in_commit(repository, &repository.find_commit(oid?)?))
            .find_map(Result::transpose)
            .transpose()
    }
}
