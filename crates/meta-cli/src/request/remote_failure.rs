use crate::request::output::Listed;

#[derive(Debug)]
pub enum RemoteFailure {
    NotGithub(Listed),
    GitFailed(Listed),
}

impl RemoteFailure {
    pub fn into_listed(self) -> Listed {
        match self {
            Self::NotGithub(reason) | Self::GitFailed(reason) => reason,
        }
    }

    pub fn retry(self) -> Result<Listed, Listed> {
        match self {
            Self::GitFailed(reason) => Ok(reason),
            Self::NotGithub(reason) => Err(reason),
        }
    }
}
