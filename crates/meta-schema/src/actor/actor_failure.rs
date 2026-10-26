use thiserror::Error;

use crate::unparsed::Unparsed;

#[derive(Debug, Error)]
pub enum ActorFailure {
    #[error("`{0}` is not an actor name: lowercase letters, digits, - _ .")]
    NotASlug(Unparsed),
}
