use super::change::Change;
use super::look::Look;
use super::made::Made;
use super::seen::Seen;

// One entry of a decision's journal: what was asked, and what the world answered, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Turn {
    Looked(Vec<Look>, Vec<Seen>),
    Acted(Vec<Change>, Vec<Made>),
}
