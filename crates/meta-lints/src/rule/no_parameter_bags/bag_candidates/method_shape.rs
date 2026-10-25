use syn::{Expr, ImplItemFn, Stmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MethodShape {
    Constructor,
    Accessor,
    Updater,
    Logic,
}

impl MethodShape {
    pub fn classify(function: &ImplItemFn) -> MethodShape {
        match function.block.stmts.as_slice() {
            [Stmt::Expr(Expr::Field(field), None)] if matches!(&*field.base, Expr::Path(base) if base.path.is_ident("self")) => {
                MethodShape::Accessor
            }
            [Stmt::Expr(Expr::Struct(literal), None)] if literal.rest.is_some() => MethodShape::Updater,
            [Stmt::Expr(Expr::Struct(_), None)] => MethodShape::Constructor,
            _ => MethodShape::Logic,
        }
    }
}
