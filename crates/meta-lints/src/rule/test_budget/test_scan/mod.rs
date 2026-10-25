mod test_module;
mod tests_over;

use test_module::TestModule;
use tests_over::TestsOver;

use std::mem;

use syn::ItemMod;
use syn::visit::Visit;

use super::TestBudget;
use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation::Violation;

pub(super) struct TestScan<'scan> {
    path: &'scan str,
    budget: TestBudget,
}

impl<'scan> TestScan<'scan> {
    pub(super) fn budget(path: &'scan str, file: &syn::File) -> TestBudget {
        let mut scan = TestScan { path, budget: TestBudget::default() };
        scan.visit_file(file);
        let over = (scan.budget.tests > TestBudget::MAX_TESTS_PER_FILE).then(|| {
            Violation::new(
                Site::File(path.to_owned()),
                TestsOver::new(scan.budget.tests, TestBudget::MAX_TESTS_PER_FILE),
            )
        });
        scan.budget + TestBudget { violations: over.into_iter().collect(), ..TestBudget::default() }
    }
}

impl<'ast> Visit<'ast> for TestScan<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if SourceFile::is_test_item(&module.attrs) {
            self.budget = mem::take(&mut self.budget) + TestModule::budget(self.path, module);
        } else {
            syn::visit::visit_item_mod(self, module);
        }
    }
}
