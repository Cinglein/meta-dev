use std::convert::Infallible;
use std::str::FromStr;

use super::YamlNode;
use crate::task::adoption::scalar::Scalar;

#[derive(Debug, Default)]
pub struct YamlLine {
    indent_len: usize,
    key: String,
    value: String,
}

impl YamlLine {
    const ITEM: &str = "- ";
    const PAIR: &str = ": ";
    const COMMENT: &str = "#";
    const BLOCK: [char; 2] = ['|', '>'];
    const FLOW: [char; 2] = ['[', ']'];
    const SEPARATORS: [char; 3] = [',', '"', '\''];

    pub fn lines(text: &str) -> Vec<YamlLine> {
        text.lines().fold(Vec::new(), |mut lines, raw| {
            let Ok(line) = raw.parse::<YamlLine>();
            let added = match lines.last_mut().filter(|open| open.holds(&line)) {
                Some(open) => open.continued(raw),
                None => line.entries(),
            };
            lines.extend(added);
            lines
        })
    }

    pub fn indent_len(&self) -> usize {
        self.indent_len
    }

    pub fn node(&self, children: Vec<YamlNode>) -> YamlNode {
        YamlNode { key: self.key.clone(), scalar: Scalar::from(self.value.clone()), children }
    }

    fn holds(&self, line: &YamlLine) -> bool {
        self.value.starts_with(Self::BLOCK) && (line.key.is_empty() || line.indent_len > self.indent_len)
    }

    fn continued(&mut self, raw: &str) -> Vec<YamlLine> {
        self.value.extend(["\n", raw.trim()]);
        Vec::new()
    }

    fn entries(self) -> Vec<YamlLine> {
        let YamlLine { indent_len, key, value } = self;
        let item = YamlNode::ITEM.to_owned();
        match key.strip_prefix(Self::ITEM) {
            _ if key.is_empty() || key.starts_with(Self::COMMENT) => Vec::new(),
            Some(rest) if !rest.contains(Self::PAIR) && !rest.ends_with(':') => {
                vec![YamlLine { indent_len, key: item, value: rest.to_owned() }]
            }
            Some(rest) => {
                let Ok(inner) = format!("{}{rest}", " ".repeat(indent_len + Self::ITEM.len())).parse::<YamlLine>();
                Some(YamlLine { indent_len, key: item, value }).into_iter().chain(inner.entries()).collect()
            }
            None if key == YamlNode::ITEM => vec![YamlLine { indent_len, key, value }],
            None => YamlLine { indent_len, key, value }.pair(),
        }
    }

    fn pair(&self) -> Vec<YamlLine> {
        let indent_len = self.indent_len;
        let text = self.key.as_str();
        let (key, value) = text.split_once(Self::PAIR).map_or_else(
            || (text.strip_suffix(':').unwrap_or(text).to_owned(), String::new()),
            |(key, value)| (key.to_owned(), value.trim().to_owned()),
        );
        let flow = value.strip_prefix(Self::FLOW[0]).and_then(|value| value.strip_suffix(Self::FLOW[1]));
        let items: Vec<YamlLine> = flow
            .into_iter()
            .flat_map(|items| items.split(','))
            .map(|item| YamlLine {
                indent_len: indent_len + Self::ITEM.len(),
                key: item.trim().trim_matches(Self::SEPARATORS).to_owned(),
                value: String::new(),
            })
            .collect();
        let value = if flow.is_some() { String::new() } else { value };
        Some(YamlLine { indent_len, key, value }).into_iter().chain(items).collect()
    }
}

impl FromStr for YamlLine {
    type Err = Infallible;

    fn from_str(raw: &str) -> Result<YamlLine, Infallible> {
        let key = raw.trim().to_owned();
        Ok(YamlLine { indent_len: raw.len() - raw.trim_start().len(), key, value: String::new() })
    }
}
