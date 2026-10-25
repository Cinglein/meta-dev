mod body;
mod returns;

use body::Body;
use returns::Returns;

use syn::{FnArg, ImplItemFn, Type};

use super::declarations::Declarations;
use super::impl_context::ImplContext;
use super::type_name::TypeName;
use crate::site::Site;

pub struct FunctionShape {
    site: Site,
    returns: Returns,
    vertex_parameters: Vec<TypeName>,
    body: Body,
}

impl FunctionShape {
    pub fn describe(
        path: &str,
        function: &ImplItemFn,
        context: &ImplContext,
        declarations: &Declarations,
    ) -> FunctionShape {
        let vertex = |ty: &Type| {
            let name = declarations.resolve(TypeName::from_type(ty).or_self(context.self_type()));
            declarations.is_vertex(&name).then_some(name)
        };
        FunctionShape {
            site: Site::spanned(path, function.sig.ident.span()),
            returns: Returns::classify(&function.sig.output, vertex),
            vertex_parameters: function
                .sig
                .inputs
                .iter()
                .filter_map(|argument| match argument {
                    FnArg::Typed(typed) => vertex(&typed.ty),
                    FnArg::Receiver(_) => None,
                })
                .collect(),
            body: Body::classify(&function.block),
        }
    }

    pub fn site(&self) -> &Site {
        &self.site
    }

    pub fn returns(&self) -> &Returns {
        &self.returns
    }

    pub fn vertex_parameters(&self) -> &[TypeName] {
        &self.vertex_parameters
    }

    pub fn projects_field(&self) -> bool {
        self.body == Body::FieldProjection
    }

    pub fn holds_vertex_in_tuple(&self) -> bool {
        self.returns == Returns::TupleHoldingVertex
    }
}
