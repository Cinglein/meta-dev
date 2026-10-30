use syn::visit::Visit;
use syn::{
    BinOp, Block, Expr, ExprAssign, ExprBinary, ExprField, ExprMethodCall, ExprPath, ImplItem, ImplItemFn, Item,
    ItemImpl, Macro,
};
use syn_rules::{FieldAccess, MacroExpressions};

use crate::field_name::FieldName;
use crate::field_write::FieldWrite;
use crate::method_key::MethodKey;
use crate::method_name::MethodName;
use crate::method_use::MethodUse;
use crate::receiver_kind::ReceiverKind;
use crate::trait_key::TraitKey;
use crate::type_key::TypeKey;
use crate::usage::Usage;
use crate::usage_node::UsageNode;
use crate::write_kind::WriteKind;

#[derive(Debug, Default)]
pub struct UsageScan {
    usage: Usage,
}

impl UsageScan {
    const SELF: &'static str = "self";
    const FILLERS: [&'static str; 5] = ["insert", "replace", "take", "get_or_insert", "get_or_insert_with"];

    #[must_use]
    pub fn scanned(block: &Block) -> Usage {
        let mut scan = UsageScan::default();
        scan.visit_block(block);
        scan.usage
    }

    #[must_use]
    pub fn methods(owner: &TypeKey, implemented: Option<&TraitKey>, item: &ItemImpl) -> Vec<MethodUse> {
        item.items
            .iter()
            .filter_map(|member| match member {
                ImplItem::Fn(function) => Self::method(owner, implemented, function),
                _ => None,
            })
            .collect()
    }

    #[must_use]
    pub fn method(owner: &TypeKey, implemented: Option<&TraitKey>, function: &ImplItemFn) -> Option<MethodUse> {
        let receiver = function.sig.receiver().map(ReceiverKind::taken_by);
        let key = MethodKey::new(implemented.cloned(), MethodName::new(&function.sig.ident));
        let accessed = FieldAccess::classify(&function.block).map(FieldAccess::member).map(FieldName::new);
        receiver.map(|receiver| {
            MethodUse::new(owner.clone(), key, receiver, Self::scanned(&function.block)).accessing(accessed)
        })
    }

    #[must_use]
    pub fn is_self(expr: &Expr) -> bool {
        matches!(expr, Expr::Path(path) if path.path.is_ident(Self::SELF))
    }

    #[must_use]
    pub fn own_field(expr: &Expr) -> Option<FieldName> {
        match expr {
            Expr::Field(field) if Self::is_self(&field.base) => Some(FieldName::new(&field.member)),
            Expr::Field(field) => Self::own_field(&field.base),
            Expr::Paren(inner) => Self::own_field(&inner.expr),
            _ => None,
        }
    }

    fn written(&mut self, place: &Expr, how: Option<WriteKind>) {
        self.usage.write(Self::own_field(place).zip(how).map(|(field, how)| FieldWrite::new(field, how)));
    }
}

impl<'ast> Visit<'ast> for UsageScan {
    fn visit_item(&mut self, _: &'ast Item) {}

    fn visit_expr_field(&mut self, field: &'ast ExprField) {
        if Self::is_self(&field.base) {
            self.usage.touch(Some(UsageNode::Field(FieldName::new(&field.member))));
        } else {
            syn::visit::visit_expr_field(self, field);
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if Self::is_self(&call.receiver) {
            self.usage.touch(Some(UsageNode::Call(MethodName::new(&call.method))));
            call.args.iter().for_each(|argument| self.visit_expr(argument));
        } else {
            self.written(
                &call.receiver,
                Self::FILLERS.iter().any(|filler| call.method == filler).then_some(WriteKind::Filled),
            );
            syn::visit::visit_expr_method_call(self, call);
        }
    }

    fn visit_expr_assign(&mut self, assign: &'ast ExprAssign) {
        self.written(&assign.left, Some(WriteKind::Assigned));
        syn::visit::visit_expr_assign(self, assign);
    }

    fn visit_expr_binary(&mut self, binary: &'ast ExprBinary) {
        let compound = matches!(
            binary.op,
            BinOp::AddAssign(_)
                | BinOp::SubAssign(_)
                | BinOp::MulAssign(_)
                | BinOp::DivAssign(_)
                | BinOp::RemAssign(_)
                | BinOp::BitXorAssign(_)
                | BinOp::BitAndAssign(_)
                | BinOp::BitOrAssign(_)
                | BinOp::ShlAssign(_)
                | BinOp::ShrAssign(_)
        );
        self.written(&binary.left, compound.then_some(WriteKind::Assigned));
        syn::visit::visit_expr_binary(self, binary);
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        self.usage.touch(path.path.is_ident(Self::SELF).then_some(UsageNode::Whole));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        MacroExpressions::visit_parsed(invocation, self);
    }
}
