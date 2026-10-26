pub(crate) struct Record { arm: Plies } impl Record { pub fn new(arm: Plies) -> Record { Record { arm } } pub fn arm(&self) -> Plies { self.arm } }
