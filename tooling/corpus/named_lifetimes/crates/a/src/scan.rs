struct Scan<'a, 'scan> { text: &'a str, path: &'scan Path, owned: &'static str } //~ named-lifetimes: 'a
impl<'b> Scan<'b, '_> { fn view<'c>(&'c self) -> &'c str { self.text } } //~ named-lifetimes: 'b //~ named-lifetimes: 'c
