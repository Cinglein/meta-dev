mod parameter;
mod struct_fields;

use std::collections::BTreeMap;

use parameter::Parameter;
use struct_fields::StructFields;
use syn::visit::Visit;
use syn::{Expr, ExprField, ImplItemFn, ItemImpl, ItemMod, ItemStruct, Stmt, Type};

use super::callee::Callee;
use super::forwarder::Forwarder;
use super::workspace_types::WorkspaceTypes;
use crate::rule::name::Name;
use crate::source_file::SourceFile;

pub struct Forwarders<'types> {
    types: &'types WorkspaceTypes,
    structs: BTreeMap<Name, StructFields>,
    self_type: Option<Name>,
    concrete: Vec<Name>,
    found: Vec<Forwarder>,
}

impl<'types> Forwarders<'types> {
    pub fn new(types: &'types WorkspaceTypes) -> Forwarders<'types> {
        Forwarders { types, structs: BTreeMap::new(), self_type: None, concrete: Vec::new(), found: Vec::new() }
    }

    pub fn found(self) -> Vec<Forwarder> {
        self.found
    }

    fn classify(&self, function: &ImplItemFn) -> Option<Forwarder> {
        let params: Vec<Parameter> = function.sig.inputs.iter().filter_map(Parameter::describe).collect();
        let callee = match function.block.stmts.as_slice() {
            [Stmt::Expr(Expr::MethodCall(call), None)]
                if self.receiver_type(&call.receiver, &params).is_some_and(|name| self.types.declares(&name))
                    && call.args.iter().all(|arg| Self::forwards(arg, &params)) =>
            {
                Some(Callee::Method(Name::from(&call.method)))
            }
            [Stmt::Expr(Expr::Call(call), None)] if call.args.iter().all(|arg| Self::forwards(arg, &params)) => {
                self.workspace_callee(&call.func)
            }
            _ => None,
        };
        callee.map(|callee| Forwarder::new(function.sig.ident.clone(), callee))
    }

    fn receiver_type(&self, expr: &Expr, params: &[Parameter]) -> Option<Name> {
        match expr {
            Expr::Path(path) if path.path.is_ident("self") => self.self_type.clone(),
            Expr::Path(path) => Parameter::named_by(params, &path.path)?.type_name().cloned(),
            Expr::Field(field) => self.field_type(field),
            Expr::Reference(reference) => self.receiver_type(&reference.expr, params),
            _ => None,
        }
    }

    fn field_type(&self, field: &ExprField) -> Option<Name> {
        match &*field.base {
            Expr::Path(base) if base.path.is_ident("self") => {
                self.structs.get(self.self_type.as_ref()?)?.field_type(&field.member, &self.concrete)
            }
            _ => None,
        }
    }

    fn workspace_callee(&self, func: &Expr) -> Option<Callee> {
        match func {
            Expr::Path(path) => Callee::associated(&path.path, self.self_type.as_ref(), self.types),
            _ => None,
        }
    }
}

impl Forwarders<'_> {
    fn forwards(expr: &Expr, params: &[Parameter]) -> bool {
        match expr {
            Expr::Path(path) => Parameter::named_by(params, &path.path).is_some(),
            Expr::Field(field) => Self::forwards(&field.base, params),
            Expr::Reference(reference) => Self::forwards(&reference.expr, params),
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for Forwarders<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Some(module)
            .filter(|module| !SourceFile::is_test_item(&module.attrs))
            .into_iter()
            .for_each(|module| syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.structs.insert(Name::from(&item.ident), StructFields::describe(item));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let segment = match &*item.self_ty {
            Type::Path(path) if item.trait_.is_none() => path.path.segments.last(),
            _ => None,
        };
        segment.into_iter().for_each(|segment| {
            self.self_type = Some(Name::from(&segment.ident));
            self.concrete = StructFields::type_arguments(segment);
            syn::visit::visit_item_impl(self, item);
        });
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.found.extend(self.classify(function));
        syn::visit::visit_impl_item_fn(self, function);
    }
}
