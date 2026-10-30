use syn::visit::Visit;
use syn::{Ident, PathSegment, Type};

use crate::interior::Interior;

#[derive(Debug, Default)]
pub struct InteriorScan {
    named: Vec<Ident>,
}

impl InteriorScan {
    const CELLS: [&'static str; 20] = [
        "Mutex",
        "RwLock",
        "RefCell",
        "Cell",
        "OnceCell",
        "OnceLock",
        "AtomicBool",
        "AtomicPtr",
        "AtomicI8",
        "AtomicI16",
        "AtomicI32",
        "AtomicI64",
        "AtomicIsize",
        "AtomicU8",
        "AtomicU16",
        "AtomicU32",
        "AtomicU64",
        "AtomicUsize",
        "LazyLock",
        "LazyCell",
    ];

    #[must_use]
    pub fn scanned(ty: &Type) -> Interior {
        let mut scan = InteriorScan::default();
        scan.visit_type(ty);
        if scan.named.iter().any(|name| Self::CELLS.iter().any(|cell| name == cell)) {
            Interior::Present
        } else {
            Interior::Absent
        }
    }
}

impl<'ast> Visit<'ast> for InteriorScan {
    fn visit_path_segment(&mut self, segment: &'ast PathSegment) {
        self.named.push(segment.ident.clone());
        syn::visit::visit_path_segment(self, segment);
    }
}
