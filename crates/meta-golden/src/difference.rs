use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Difference {
    index: usize,
    golden: Option<String>,
    actual: Option<String>,
}

impl Difference {
    const END: &'static str = "end of text";
    const LINE: char = '\n';

    pub fn between(golden: &str, actual: &str) -> Option<Self> {
        let longest_len = golden.split(Self::LINE).count().max(actual.split(Self::LINE).count());
        let pairs = Self::lines(golden).zip(Self::lines(actual)).take(longest_len);
        let (index, (golden, actual)) = pairs.enumerate().find(|(_, (golden, actual))| golden != actual)?;
        Some(Self { index, golden: golden.map(str::to_owned), actual: actual.map(str::to_owned) })
    }

    fn lines(text: &str) -> impl Iterator<Item = Option<&str>> {
        text.split(Self::LINE).map(Some).chain(std::iter::repeat(None))
    }
}

impl fmt::Display for Difference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown =
            |line: &Option<String>| line.as_ref().map_or_else(|| String::from(Self::END), |line| format!("`{line}`"));
        write!(
            formatter,
            "line {} differs: golden {}, actual {}",
            self.index + 1,
            shown(&self.golden),
            shown(&self.actual)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Difference;

    const GOLDEN: &str = "a\nb";
    const SHOWN: [(&str, &str); 2] =
        [("a\nc", "line 2 differs: golden `b`, actual `c`"), ("a", "line 2 differs: golden `b`, actual end of text")];

    #[test]
    fn the_first_differing_line_is_named_with_both_texts_and_equal_texts_have_none() {
        let shown =
            SHOWN.map(|(actual, _)| Difference::between(GOLDEN, actual).map(|difference| difference.to_string()));
        assert_eq!(shown, SHOWN.map(|(_, want)| Some(want.to_owned())));
        assert_eq!(Difference::between(GOLDEN, GOLDEN), None);
    }
}
