mod body;
mod returns;

use body::Body;
use returns::Returns;

use std::path::Path;

use syn::{FnArg, ImplItemFn, Type};

use super::declarations::Declarations;
use super::impl_context::ImplContext;
use task::LineNumber;
use task::Site;
use task::TypeName;

pub struct FunctionShape {
    site: Site,
    returns: Returns,
    vertex_parameters: Vec<TypeName>,
    body: Body,
}

impl FunctionShape {
    pub fn describe(
        path: &Path,
        function: &ImplItemFn,
        context: &ImplContext,
        declarations: &Declarations,
    ) -> FunctionShape {
        let vertex = |ty: &Type| {
            let name = declarations.resolve(TypeName::from_type(ty).or_self(context.self_type()));
            declarations.is_vertex(&name).then_some(name)
        };
        FunctionShape {
            site: Site::Line(path.into(), LineNumber::from(function.sig.ident.span())),
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

    pub fn body(&self) -> Body {
        self.body
    }
}
