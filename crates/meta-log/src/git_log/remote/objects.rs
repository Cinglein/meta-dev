mod batch_command;
mod header;

use std::io;
use std::str::FromStr;

use super::Remote;
use crate::git_log::content::Content;
use crate::git_log::sha::Sha;
use crate::log_failure::LogFailure;
use crate::remote_name::RemoteName;
use batch_command::BatchCommand;
use header::Header;

#[derive(Debug)]
pub struct Objects {
    tree: Header,
    file: Header,
    body: String,
}

impl Objects {
    const NEWLINE: char = '\n';

    pub fn request(tip: &Sha) -> Content {
        Content::from(format!("{}{}", BatchCommand::TreeInfo(tip), BatchCommand::FileContents(tip)))
    }

    pub fn content(self, remote: &RemoteName) -> Result<Option<Content>, LogFailure> {
        let held = self.tree.tree_holding_only(Remote::FILE, remote)?;
        let whole = (self.file.blob_size() == Some(self.body.len())).then_some(self.body);
        held.map(|()| whole.map(Content::from).ok_or_else(|| LogFailure::Malformed(remote.clone()))).transpose()
    }
}

impl FromStr for Objects {
    type Err = io::Error;

    fn from_str(answered: &str) -> Result<Self, Self::Err> {
        let (tree, rest) = answered.split_once(Self::NEWLINE).unwrap_or((answered, ""));
        let (file, rest) = rest.split_once(Self::NEWLINE).unwrap_or((rest, ""));
        let body = rest.strip_suffix(Self::NEWLINE).unwrap_or(rest).to_owned();
        Ok(Self { tree: tree.parse()?, file: file.parse()?, body })
    }
}
