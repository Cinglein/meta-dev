#[cfg(test)] //~ file test-budget: 4 tests, at most 3 allowed
mod tests {
    #[test] fn one() {}
    #[test] fn two() {}
    #[test] fn three() {}
    #[test]
    fn long() { //~ test-budget: fn long has 21 body lines





















    }
}
