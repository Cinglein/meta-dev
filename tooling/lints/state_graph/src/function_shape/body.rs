use syn::Block;
use syn_rules::FieldAccess;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    FieldProjection,
    Other,
}

impl Body {
    pub fn classify(block: &Block) -> Body {
        FieldAccess::classify(block)
            .and_then(FieldAccess::projection)
            .map_or(Body::Other, |_| Body::FieldProjection)
    }
}
