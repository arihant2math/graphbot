use std::{
    collections::VecDeque,
    fmt,
    ops::{Index, IndexMut, Range, RangeBounds},
};

use crate::{
    nodes::{
        Argument, Comment, ExternalLink, HTMLEntity, Heading, Node, Tag, Template, Text, Wikilink,
    },
    utils::parse_anything,
};

pub const FLAGS: regex::RegexBuilder = *regex::RegexBuilder::new("")
    .case_insensitive(true)
    .dot_matches_new_line(true)
    .unicode(true);

#[derive(Clone, Debug)]
pub struct Wikicode {
    nodes: Vec<Box<dyn Node>>,
}

impl Wikicode {
    pub const RECURSE_OTHERS: u8 = 2;

    pub fn new(nodes: Vec<Box<dyn Node>>) -> Self {
        Self { nodes }
    }

    pub fn nodes(&self) -> &Vec<Box<dyn Node>> {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut Vec<Box<dyn Node>> {
        &mut self.nodes
    }

    pub fn get(&self, index: usize) -> &Box<dyn Node> {
        &self.nodes[index]
    }

    pub fn get_slice(&self, range: std::ops::Range<usize>) -> &[Box<dyn Node>] {
        &self.nodes[range]
    }

    pub fn set(&mut self, index: usize, value: impl Into<Box<dyn Node>>) {
        self.nodes[index] = value.into();
    }

    pub fn insert(&mut self, index: usize, value: impl Into<Box<dyn Node>>) {
        self.nodes.insert(index, value.into());
    }

    pub fn append(&mut self, value: impl Into<Box<dyn Node>>) {
        self.nodes.push(value.into());
    }

    pub fn remove(&mut self, index: usize) -> Box<dyn Node> {
        self.nodes.remove(index)
    }

    pub fn filter<'a, F>(&'a self, mut predicate: F) -> Vec<&'a Box<dyn Node>>
    where
        F: FnMut(&Box<dyn Node>) -> bool,
    {
        self.nodes.iter().filter(|n| predicate(n)).collect()
    }

    pub fn ifilter<'a, F>(&'a self, mut predicate: F) -> impl Iterator<Item = &'a Box<dyn Node>>
    where
        F: FnMut(&Box<dyn Node>) -> bool + 'a,
    {
        self.nodes.iter().filter(move |n| predicate(n))
    }

    pub fn index_of(&self, obj: &dyn Node) -> Option<usize> {
        self.nodes.iter().position(|n| n.as_ref().equals(obj))
    }

    pub fn contains_node(&self, obj: &dyn Node) -> bool {
        self.nodes.iter().any(|n| n.as_ref().equals(obj))
    }

    pub fn strip_code(
        &self,
        normalize: bool,
        collapse: bool,
        keep_template_params: bool,
    ) -> String {
        let mut nodes = Vec::new();
        for node in &self.nodes {
            if let Some(stripped) = node.strip(normalize, collapse, keep_template_params) {
                nodes.push(stripped);
            }
        }
        let mut result = nodes.join("");
        if collapse {
            while result.contains("\n\n\n") {
                result = result.replace("\n\n\n", "\n\n");
            }
            result.trim_matches('\n').to_string()
        } else {
            result
        }
    }

    pub fn get_tree(&self) -> String {
        let marker = std::sync::Arc::new(());
        let mut lines = Vec::new();
        self._get_tree(self, &mut lines, &marker, 0);
        lines.join("\n")
    }

    fn _get_tree<'a>(
        &'a self,
        code: &'a Wikicode,
        lines: &mut Vec<String>,
        marker: &std::sync::Arc<()>,
        indent: usize,
    ) {
        let write = |args: &[&str]| {
            if let Some(last) = lines.last_mut() {
                if last == &format!("{:?}", marker) {
                    lines.pop();
                    if let Some(last_line) = lines.pop() {
                        lines.push(format!("{} {}", last_line, args.join(" ")));
                    }
                } else {
                    lines.push(format!("{}{}", " ".repeat(6 * indent), args.join(" ")));
                }
            } else {
                lines.push(format!("{}{}", " ".repeat(6 * indent), args.join(" ")));
            }
        };
        let get = |code: &Wikicode| self._get_tree(code, lines, marker, indent + 1);
        let mark = || lines.push(format!("{:?}", marker));
        for node in &code.nodes {
            node.showtree(&write, &get, &mark);
        }
    }

    pub fn as_str(&self) -> String {
        self.nodes.iter().map(|n| n.as_str()).collect()
    }
}

// Implement Display for Wikicode
impl fmt::Display for Wikicode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in &self.nodes {
            write!(f, "{}", node)?;
        }
        Ok(())
    }
}

// Macro for filter methods for node types
macro_rules! build_filter_methods {
    ($($name:ident => $ty:ty),*) => {
        impl Wikicode {
            $(
                pub fn ifilter_$name(&self) -> Vec<&Box<dyn Node>> {
                    self.filter(|n| n.as_any().is::<$ty>())
                }
                pub fn filter_$name(&self) -> Vec<&Box<dyn Node>> {
                    self.filter(|n| n.as_any().is::<$ty>())
                }
            )*
        }
    };
}

build_filter_methods!(
    arguments => Argument,
    comments => Comment,
    external_links => ExternalLink,
    headings => Heading,
    html_entities => HTMLEntity,
    tags => Tag,
    templates => Template,
    text => Text,
    wikilinks => Wikilink
);
