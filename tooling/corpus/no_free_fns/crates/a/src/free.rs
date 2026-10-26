fn main() {} fn loose() {} //~ no-free-fns: fn loose //~ workspace test-budget: has 1 tests across the workspace //~ workspace test-budget: has 1 test lines
struct S; impl S { fn method(&self) {} fn associated() {} }
trait T { fn required(); fn provided() {} }
mod inner { fn nested() {} #[cfg(test)] mod tests { #[test] fn a_test() {} fn test_helper() {} } } //~ no-free-fns: fn nested //~ no-free-fns: fn test_helper
