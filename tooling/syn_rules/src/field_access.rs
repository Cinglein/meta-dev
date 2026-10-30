use syn::{Block, Expr, ExprField, Member, Stmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldAccess<'body> {
    Projected(&'body Member),
    Lent(&'body Member),
    Assigned(&'body Member),
}

impl<'body> FieldAccess<'body> {
    const SELF: &'static str = "self";

    #[must_use]
    pub fn classify(block: &'body Block) -> Option<FieldAccess<'body>> {
        match block.stmts.as_slice() {
            [Stmt::Expr(Expr::Field(field), None)] => Self::own(field).map(FieldAccess::Projected),
            [Stmt::Expr(Expr::Reference(reference), None)] => match &*reference.expr {
                Expr::Field(field) => Self::own(field).map(FieldAccess::Lent),
                _ => None,
            },
            [Stmt::Expr(Expr::Assign(assign), _)] => match &*assign.left {
                Expr::Field(field) => Self::own(field).map(FieldAccess::Assigned),
                _ => None,
            },
            _ => None,
        }
    }

    #[must_use]
    pub fn projection(self) -> Option<&'body Member> {
        match self {
            FieldAccess::Projected(member) => Some(member),
            FieldAccess::Lent(_) | FieldAccess::Assigned(_) => None,
        }
    }

    #[must_use]
    pub fn member(self) -> &'body Member {
        match self {
            FieldAccess::Projected(member) | FieldAccess::Lent(member) | FieldAccess::Assigned(member) => member,
        }
    }

    fn own(field: &'body ExprField) -> Option<&'body Member> {
        matches!(&*field.base, Expr::Path(base) if base.path.is_ident(Self::SELF)).then_some(&field.member)
    }
}
