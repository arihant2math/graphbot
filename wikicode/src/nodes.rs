use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerAttribute {
    pad_after_eq: String,
    pad_before_eq: String,
    pad_first: String,
    quotes: Option<String>,
    value: Option<Wikicode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerParameter {
    pub key: Wikicode,
    pub showkey: bool,
    pub value: Option<Wikicode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerArgument {
    name: Wikicode,
    default: Option<Wikicode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerComment {
    pub contents: Wikicode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerExternalLink {
    pub(crate) brackets: String,
    pub(crate) suppress_space: bool,
    pub(crate) title: Option<Wikicode>,
    pub(crate) url: Wikicode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerHTMLEntity {
    hex_char: String,
    hexadecimal: bool,
    named: bool,
    value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerHeading {
    pub level: u8,
    pub title: Wikicode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerTag {
    attributes: Vec<Node<NodeInnerAttribute>>,
    closing_tag: Option<String>,
    closing_wiki_markup: Option<String>,
    contents: Wikicode,
    implicit: bool,
    invalid: bool,
    padding: String,
    self_closing: bool,
    tag: Option<String>,
    wiki_markup: Option<String>,
    wiki_style_separator: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerTemplate {
    pub name: Wikicode,
    pub params: Vec<Node<NodeInnerParameter>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerText {
    pub(crate) value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerWikilink {
    // TODO: somehow rename/fix this
    #[serde(rename = "txt")]
    pub text: Option<Wikicode>,
    pub title: Wikicode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node<I>
where
    I: Clone + Debug,
{
    #[serde(flatten)]
    pub(crate) inner: I,
}

impl<T> Deref for Node<T>
where
    T: Clone + Debug,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Node<T>
where
    T: Clone + Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeInner {
    Attribute(NodeInnerAttribute),
    Parameter(NodeInnerParameter),
    Argument(NodeInnerArgument),
    Comment(NodeInnerComment),
    ExternalLink(NodeInnerExternalLink),
    Heading(NodeInnerHeading),
    HTMLEntity(NodeInnerHTMLEntity),
    Tag(NodeInnerTag),
    Template(NodeInnerTemplate),
    Text(NodeInnerText),
    Wikilink(NodeInnerWikilink),
}

use paste::paste;

macro_rules! as_part {
    ($name:ident, $variant:ident, $st:ident) => {
        paste! {
            impl NodeInner {
                #[allow(unused)]
                pub fn [<as_ $name>](&self) -> Option<&$st> {
                    if let NodeInner::$variant(inner) = self {
                        Some(inner)
                    } else {
                        None
                    }
                }

                #[allow(unused)]
                pub fn [<into_ $name>](self) -> Option<$st> {
                    if let NodeInner::$variant(inner) = self {
                        Some(inner)
                    } else {
                        None
                    }
                }

                #[allow(unused)]
                pub fn [<is_ $name>](&self) -> bool {
                    matches!(self, NodeInner::$variant(_))
                }
            }

            impl GenericNode {
                #[allow(unused)]
                pub fn [<into_ $name>](self) -> Option<Node<$st>> {
                    if let NodeInner::$variant(inner) = self.inner {
                        Some(Node {
                            inner,
                        })
                    } else {
                        None
                    }
                }

                #[allow(unused)]
                pub fn [<is_ $name>](&self) -> bool {
                    matches!(self.inner, NodeInner::$variant(_))
                }
            }
        }
    };
}

as_part!(attribute, Attribute, NodeInnerAttribute);
as_part!(parameter, Parameter, NodeInnerParameter);
as_part!(argument, Argument, NodeInnerArgument);
as_part!(comment, Comment, NodeInnerComment);
as_part!(external_link, ExternalLink, NodeInnerExternalLink);
as_part!(heading, Heading, NodeInnerHeading);
as_part!(html_entity, HTMLEntity, NodeInnerHTMLEntity);
as_part!(tag, Tag, NodeInnerTag);
as_part!(template, Template, NodeInnerTemplate);
as_part!(text, Text, NodeInnerText);
as_part!(wikilink, Wikilink, NodeInnerWikilink);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenericNode {
    #[serde(flatten)]
    pub inner: NodeInner,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wikicode {
    pub nodes: Vec<GenericNode>,
}

impl Wikicode {
    pub fn new() -> Self {
        Wikicode { nodes: Vec::new() }
    }

    pub fn from_nodes(nodes: Vec<GenericNode>) -> Self {
        Wikicode { nodes }
    }
}
