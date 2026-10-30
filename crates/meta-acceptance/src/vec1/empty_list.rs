use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("the list needs at least one entry")]
pub struct EmptyList;

impl EmptyList {
    pub fn refused<Item>(items: Vec<Item>) -> Result<Vec<Item>, EmptyList> {
        (!items.is_empty()).then_some(items).ok_or(EmptyList)
    }
}
