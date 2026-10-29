mod yaml_line;

use std::convert::Infallible;
use std::str::FromStr;

use crate::task::adoption::scalar::Scalar;
use yaml_line::YamlLine;

#[derive(Debug, Default)]
pub struct YamlNode {
    key: String,
    scalar: Scalar,
    children: Vec<YamlNode>,
}

impl YamlNode {
    pub const ITEM: &str = "-";

    pub fn child(&self, key: &str) -> Option<&YamlNode> {
        self.children.iter().find(|child| child.key == key)
    }

    pub fn items(&self, key: &str) -> &[YamlNode] {
        self.child(key).map_or(&[], |child| &child.children)
    }

    pub fn scalar(&self) -> &Scalar {
        &self.scalar
    }

    pub fn scalar_of(&self, key: &str) -> &str {
        self.child(key).map_or("", |child| &child.scalar)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.children.iter().map(YamlNode::name).chain(Some(&*self.scalar).filter(|scalar| !scalar.is_empty()))
    }

    pub fn name(&self) -> &str {
        if self.key == Self::ITEM { &self.scalar } else { &self.key }
    }

    fn nested(lines: &[YamlLine]) -> Vec<YamlNode> {
        lines.split_first().map_or_else(Vec::new, |(first, tail)| {
            let end = tail.iter().position(|line| line.indent_len() <= first.indent_len()).unwrap_or(tail.len());
            let (inner, rest) = tail.split_at(end);
            Some(first.node(Self::nested(inner))).into_iter().chain(Self::nested(rest)).collect()
        })
    }
}

impl FromStr for YamlNode {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<YamlNode, Infallible> {
        Ok(YamlNode { children: Self::nested(&YamlLine::lines(text)), ..YamlNode::default() })
    }
}
