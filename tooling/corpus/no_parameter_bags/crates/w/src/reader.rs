pub struct Reader; impl Reader { pub fn read(&self, record: Record) -> Plies { record.arm() + 1 } }
