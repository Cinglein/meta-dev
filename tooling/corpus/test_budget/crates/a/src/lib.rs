#[cfg(test)] mod fixtures; //~ workspace test-budget: has 7 tests across the workspace, at most 1 allowed //~ workspace test-budget: has 42 test lines, at most 7 allowed
#[cfg(test)]
mod tests {
    #[test]
    fn example() { let squares = [Square::A1, Square::H8]; assert_eq!(squares.len(), 2); assert!(Bitboard::EMPTY.is_empty()); assert_ne!("a", "b"); assert_eq!(seed(), 0x2545_F491); } //~ test-budget: fn example has 4 assertions //~ test-budget: fn example has 6 literals //~ test-budget: integer literal 0x2545_F491
    #[test]
    fn property() { proptest!(|(count in 0..100usize, flag: bool)| { prop_assert!(count < 100 || flag); }); } //~ test-budget: integer literal 100usize //~ test-budget: integer literal 100 in
    #[test] fn edge() { assert!(true); assert!(true); assert_eq!(64, 64); }
    fn helper() -> Vec<Plies> { assert!(true); vec![1] } //~ test-budget: assertion outside a test //~ no-free-fns: fn helper
}
