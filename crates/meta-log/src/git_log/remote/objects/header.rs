use std::io;
use std::str::FromStr;

use crate::git_log::sha::Sha;
use crate::log_failure::LogFailure;
use crate::remote_name::RemoteName;

#[derive(Debug, PartialEq, Eq)]
pub enum Header {
    Blob { size: usize },
    Tree { sha: Sha, size: usize },
    Missing,
}

impl Header {
    const MODE_LENGTH: usize = 6;
    const BLOB: &'static str = "blob";
    const TREE: &'static str = "tree";

    pub fn tree_holding_only(&self, file: &str, remote: &RemoteName) -> Result<Option<()>, LogFailure> {
        match self {
            Self::Missing => Ok(None),
            Self::Tree { sha, size } if *size == Self::MODE_LENGTH + 1 + file.len() + 1 + sha.digest_length() => {
                Ok(Some(()))
            }
            Self::Tree { .. } | Self::Blob { .. } => Err(LogFailure::Malformed(remote.clone())),
        }
    }

    pub fn blob_size(&self) -> Option<usize> {
        match self {
            Self::Blob { size } => Some(*size),
            Self::Tree { .. } | Self::Missing => None,
        }
    }
}

impl FromStr for Header {
    type Err = io::Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let size = |size: &str| size.parse().map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
        match line.split_whitespace().collect::<Vec<_>>()[..] {
            [_, Self::BLOB, blob] => Ok(Self::Blob { size: size(blob)? }),
            [sha, Self::TREE, tree] => Ok(Self::Tree {
                sha: sha.parse().map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
                size: size(tree)?,
            }),
            _ => Ok(Self::Missing),
        }
    }
}
