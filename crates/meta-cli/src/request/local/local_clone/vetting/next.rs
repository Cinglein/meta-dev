use super::change::Change;
use super::look::Look;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Next<Answer> {
    Look(Vec<Look>),
    Act(Vec<Change>),
    Done(Answer),
}
