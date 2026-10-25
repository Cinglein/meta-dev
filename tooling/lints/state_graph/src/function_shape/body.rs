use syn::{Block, Expr, Stmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    FieldProjection,
    Other,
}

impl Body {
    pub fn classify(block: &Block) -> Body {
        match block.stmts.as_slice() {
            [Stmt::Expr(Expr::Field(field), None)] if matches!(&*field.base, Expr::Path(base) if base.path.is_ident("self")) => {
                Body::FieldProjection
            }
            _ => Body::Other,
        }
    }

    pub fn projects_field(self) -> bool {
        matches!(self, Body::FieldProjection)
    }
}
