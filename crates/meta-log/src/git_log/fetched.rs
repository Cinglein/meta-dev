use std::str::FromStr;

use super::content::Content;
use super::sha::Sha;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fetched {
    tip: Option<Sha>,
    content: Content,
}

impl Fetched {
    const MISSING: &'static str = " missing";

    pub fn tip(&self) -> Option<&Sha> {
        self.tip.as_ref()
    }

    pub fn content(&self) -> &Content {
        &self.content
    }
}

impl FromStr for Fetched {
    type Err = std::convert::Infallible;

    fn from_str(shown: &str) -> Result<Self, Self::Err> {
        let (header, rest) = shown.split_once('\n').unwrap_or((shown, ""));
        let tip = Some(header)
            .filter(|header| !header.ends_with(Self::MISSING))
            .and_then(|header| header.split_whitespace().next())
            .map(|sha| sha.parse().unwrap_or_else(|never| match never {}));
        let content = rest
            .split_once('\n')
            .map(|(_, text)| text)
            .map(|text| Content::from(text.to_owned()).terminated())
            .unwrap_or_default();
        Ok(Self { content: tip.as_ref().map_or_else(Content::default, |_| content), tip })
    }
}

#[cfg(test)]
mod tests {
    use super::Fetched;

    const SHOWN: &str = "c0ffee commit 180\nbeef blob 12\n{\"a\":1}\n{\"b\":2}";
    const NONE: &str =
        "refs/remotes/origin/meta/log^{commit} missing\nrefs/remotes/origin/meta/log:events.jsonl missing";
    const CONTENT: &str = "{\"a\":1}\n{\"b\":2}\n";
    const TIP: &str = "c0ffee";

    #[test]
    fn a_batch_listing_gives_the_tip_and_the_terminated_log_and_a_missing_branch_gives_neither() {
        let shown: Fetched = SHOWN.parse().unwrap();
        let none: Fetched = NONE.parse().unwrap();
        assert_eq!(
            (shown.tip().map(ToString::to_string), shown.content().to_string()),
            (Some(TIP.to_owned()), CONTENT.to_owned())
        );
        assert_eq!((none.tip(), none.content().to_string()), (None, String::new()));
    }
}
