pub trait VerdictVisitor {
    type Output;

    fn instrumented(self) -> Self::Output;
    fn uninstrumented(self) -> Self::Output;
    fn on_target(self) -> Self::Output;
    fn failing(self) -> Self::Output;
}
