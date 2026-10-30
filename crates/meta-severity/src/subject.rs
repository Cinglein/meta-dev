use meta_schema::{ItemKind, Placement};

pub trait Subject {
    fn placement(&self) -> Placement;
    fn kind(&self) -> ItemKind;
}
