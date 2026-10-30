use crate::git_failure::GitFailure;
use crate::query::Query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitEmails(String);

impl CommitEmails {
    const HEAD: &'static str = "HEAD";
}

impl From<&str> for CommitEmails {
    fn from(base: &str) -> CommitEmails {
        CommitEmails(base.to_owned())
    }
}

impl Query for CommitEmails {
    type View = Vec<String>;

    fn answered(&self, repository: &git2::Repository) -> Result<Vec<String>, GitFailure> {
        let mut walk = repository.revwalk()?;
        walk.push_ref(Self::HEAD)?;
        walk.hide(repository.revparse_single(&self.0)?.peel_to_commit()?.id())?;
        let emails = walk.map(|oid| {
            let commit = repository.find_commit(oid?)?;
            let email = |signature: git2::Signature<'_>| String::from_utf8_lossy(signature.email_bytes()).into_owned();
            Ok(vec![email(commit.author()), email(commit.committer())])
        });
        Ok(emails.collect::<Result<Vec<Vec<String>>, git2::Error>>()?.into_iter().flatten().collect())
    }
}
