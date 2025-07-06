use std::fmt::Debug;

pub trait Node: Debug {}

#[derive(Debug)]
pub struct AttributeNode {
    pub pad_after_eq: String,
    pub pad_before_eq: String,
    pub pad_first: String,
    pub quotes: Option<String>,
    pub value: Option<Wikicode>,
}

impl Node for AttributeNode {}

#[derive(Debug)]
pub struct ParameterNode {
    pub key: Wikicode,
    pub showkey: bool,
    pub value: Option<Wikicode>,
}

impl Node for ParameterNode {}

#[derive(Debug)]
pub struct ArgumentNode {
    pub name: Wikicode,
    pub default: Option<Wikicode>,
}

impl Node for ArgumentNode {}

#[derive(Debug)]
pub struct CommentNode {
    pub contents: Wikicode,
}

impl Node for CommentNode {}

#[derive(Debug)]
pub struct ExternalLinkNode {
    pub brackets: String,
    pub suppress_space: bool,
    pub title: Option<Wikicode>,
    pub url: Wikicode,
}

impl Node for ExternalLinkNode {}

#[derive(Debug)]
pub struct HTMLEntityNode {
    pub hex_char: String,
    pub hexadecimal: bool,
    pub named: bool,
    pub value: String,
}

impl Node for HTMLEntityNode {}

#[derive(Debug)]
pub struct HeadingNode {
    pub level: u8,
    pub title: Wikicode,
}

impl Node for HeadingNode {}

#[derive(Debug)]
pub struct TagNode {
    pub attributes: Vec<AttributeNode>,
    pub closing_tag: Option<String>,
    pub closing_wiki_markup: Option<String>,
    pub contents: Wikicode,
    pub implicit: bool,
    pub invalid: bool,
    pub padding: String,
    pub self_closing: bool,
    pub tag: Option<String>,
    pub wiki_markup: Option<String>,
    pub wiki_style_separator: Option<String>,
}

impl Node for TagNode {}

#[derive(Debug)]
pub struct TemplateNode {
    pub name: Wikicode,
    pub params: Vec<ParameterNode>,
}

impl Node for TemplateNode {}

#[derive(Clone, Debug)]
pub struct TextNode {
    pub value: String,
}

impl Node for TextNode {}

#[derive(Debug)]
pub struct WikilinkNode {
    pub text: Option<Wikicode>,
    pub title: Wikicode,
}

impl Node for WikilinkNode {}

#[derive(Debug)]
pub struct Wikicode {
    pub nodes: Vec<Box<dyn Node>>,
}

impl Wikicode {
    pub fn new() -> Self {
        Wikicode { nodes: Vec::new() }
    }

    pub fn from_nodes(nodes: Vec<Box<dyn Node>>) -> Self {
        Wikicode { nodes }
    }
}
