use std::str::FromStr;

use crate::gh_field::GhField;
use crate::gh_key::GhKey;
use task::Failure;

pub struct Repository {
    owner: String,
    name: String,
}

impl Repository {
    const CUT: char = '/';
    const FIELDS_LENGTH: usize = 2;

    pub fn fields(&self) -> [GhField; Self::FIELDS_LENGTH] {
        [GhField::new(GhKey::Owner, &self.owner), GhField::new(GhKey::Name, &self.name)]
    }
}

impl FromStr for Repository {
    type Err = Failure;

    fn from_str(text: &str) -> Result<Repository, Failure> {
        let (owner, name) =
            text.split_once(Self::CUT).ok_or_else(|| Failure::Usage(format!("{text} is not owner/name")))?;
        Ok(Repository { owner: owner.to_owned(), name: name.to_owned() })
    }
}

#[cfg(test)]
mod tests {
    use super::Repository;
    #[test]
    fn a_repository_splits_into_the_owner_and_name_fields_gh_takes() {
        let fields = "owner/project".parse::<Repository>().unwrap().fields();
        let broken = "project".parse::<Repository>().is_err();
        assert!(fields.map(|field| field.to_string()) == ["owner=owner", "name=project"] && broken);
    }
}
