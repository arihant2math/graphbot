use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use paste::paste;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::config::Config;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerAttribute {
    pad_after_eq: String,
    pad_before_eq: String,
    pad_first: String,
    quotes: Option<String>,
    value: Option<Wikitext>,
}

impl NodeInnerAttribute {
    pub fn pad_after_eq(&self) -> &str {
        &self.pad_after_eq
    }

    pub fn pad_before_eq(&self) -> &str {
        &self.pad_before_eq
    }

    pub fn pad_first(&self) -> &str {
        &self.pad_first
    }

    pub fn quotes(&self) -> Option<&str> {
        self.quotes.as_deref()
    }

    pub fn value_str(&self) -> Option<&str> {
        self.value.as_ref().map(|v| &*v.text)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerParameter {
    pub name: String,
    pub showkey: bool,
    pub value: Option<String>,
}

impl NodeInnerParameter {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn showkey(&self) -> bool {
        self.showkey
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerArgument {
    name: Wikitext,
    default: Option<Wikitext>,
}

impl NodeInnerArgument {
    pub fn name(&self) -> &Wikitext {
        &self.name
    }

    pub fn default(&self) -> Option<&Wikitext> {
        self.default.as_ref()
    }

    pub fn name_str(&self) -> &str {
        &self.name.text
    }

    pub fn default_str(&self) -> Option<&str> {
        self.default.as_ref().map(|d| &*d.text)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerComment {
    contents: String,
}

impl NodeInnerComment {
    pub fn contents(&self) -> &str {
        &self.contents
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerExternalLink {
    brackets: bool,
    title: Option<Wikitext>,
    url: Wikitext,
}

impl NodeInnerExternalLink {
    pub fn brackets(&self) -> bool {
        self.brackets
    }

    pub fn title(&self) -> Option<&Wikitext> {
        self.title.as_ref()
    }

    pub fn title_str(&self) -> Option<&str> {
        self.title.as_ref().map(|t| &*t.text)
    }

    pub fn url(&self) -> &Wikitext {
        &self.url
    }

    pub fn url_str(&self) -> &str {
        &self.url.text
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerHTMLEntity {
    hex_char: String,
    hexadecimal: bool,
    named: bool,
    value: String,
}

impl NodeInnerHTMLEntity {
    pub fn hex_char(&self) -> &str {
        &self.hex_char
    }

    pub fn hexadecimal(&self) -> bool {
        self.hexadecimal
    }

    pub fn named(&self) -> bool {
        self.named
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerHeading {
    level: u8,
    title: Wikitext,
}

impl NodeInnerHeading {
    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn title(&self) -> &Wikitext {
        &self.title
    }

    pub fn title_str(&self) -> &str {
        &self.title.text
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerTag {
    attributes: Vec<Node<NodeInnerAttribute>>,
    closing_tag: Option<String>,
    closing_wiki_markup: Option<String>,
    contents: Wikitext,
    implicit: bool,
    invalid: bool,
    padding: String,
    self_closing: bool,
    tag: Option<String>,
    wiki_markup: Option<String>,
    wiki_style_separator: Option<String>,
}

impl NodeInnerTag {
    pub fn attributes(&self) -> &[Node<NodeInnerAttribute>] {
        &self.attributes
    }

    pub fn closing_tag(&self) -> Option<&str> {
        self.closing_tag.as_deref()
    }

    pub fn closing_wiki_markup(&self) -> Option<&str> {
        self.closing_wiki_markup.as_deref()
    }

    pub fn contents(&self) -> &Wikitext {
        &self.contents
    }

    pub fn contents_str(&self) -> &str {
        &self.contents.text
    }

    pub fn implicit(&self) -> bool {
        self.implicit
    }

    pub fn invalid(&self) -> bool {
        self.invalid
    }

    pub fn padding(&self) -> &str {
        &self.padding
    }

    pub fn self_closing(&self) -> bool {
        self.self_closing
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    pub fn wiki_markup(&self) -> Option<&str> {
        self.wiki_markup.as_deref()
    }

    pub fn wiki_style_separator(&self) -> Option<&str> {
        self.wiki_style_separator.as_deref()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerTemplate {
    name: Wikitext,
    pub params: Vec<Node<NodeInnerParameter>>,
}

impl NodeInnerTemplate {
    pub fn name(&self) -> &Wikitext {
        &self.name
    }

    pub fn params(&self) -> &[Node<NodeInnerParameter>] {
        &self.params
    }

    pub fn params_map(&self) -> HashMap<String, Option<String>> {
        self.params
            .iter()
            .map(|param| {
                (
                    param.name().to_string(),
                    param.value().map(|s| s.to_string()),
                )
            })
            .collect()
    }

    pub fn params_insert(&mut self, name: String, value: Option<String>) {
        self.params.push(Node {
            text: name.clone(),
            inner: NodeInnerParameter {
                name,
                showkey: true,
                value,
            },
        });
    }

    pub fn params_get(&self, name: &str) -> Option<Option<String>> {
        self.params
            .iter()
            .find(|param| param.name() == name)
            .map(|param| param.value().map(|s| s.to_string()))
    }

    pub fn params_remove(&mut self, name: &str) {
        self.params.retain(|param| param.name() != name);
    }

    pub fn name_str(&self) -> &str {
        &self.name.text
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerText {
    value: String,
}

impl NodeInnerText {
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInnerWikilink {
    // TODO: somehow rename/fix this
    #[serde(rename = "txt")]
    text: Option<Wikitext>,
    title: Wikitext,
}

impl NodeInnerWikilink {
    pub fn text(&self) -> Option<&Wikitext> {
        self.text.as_ref()
    }

    pub fn text_str(&self) -> Option<&str> {
        self.text.as_ref().map(|t| &*t.text)
    }

    pub fn title(&self) -> &Wikitext {
        &self.title
    }

    pub fn title_str(&self) -> &str {
        &self.title.text
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node<I>
where
    I: Clone + Debug,
{
    pub text: String,
    #[serde(flatten)]
    inner: I,
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
                            text: self.text,
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
    pub text: String,
    #[serde(flatten)]
    pub inner: NodeInner,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wikitext {
    // pub headings: Vec<Node<NodeInnerHeading>>,
    pub templates: Vec<Node<NodeInnerTemplate>>,
    pub tags: Vec<Node<NodeInnerTag>>,
    pub nodes: Vec<GenericNode>,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutRoot {
    pub parsed: Wikitext,
    pub elapsed: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XMLResponse {
    pub parsed: String,
    pub elapsed: f64,
}

pub async fn call_parser(input: &str, config: &RwLock<Config>) -> anyhow::Result<OutRoot> {
    let mut client = xml_rpc::Client::new().unwrap();
    let config = config.read().await;
    let result = client.call(
        &xml_rpc::Url::parse(&format!(
            "http://{}:{}/{}",
            config.rpc.host, config.rpc.port, config.rpc.path
        ))?,
        "parse",
        [input],
    );
    let response: XMLResponse = result
        .map_err(|e| anyhow::anyhow!("XML-RPC call failed: {}", e))
        .and_then(|response| {
            response.map_err(|e| anyhow::anyhow!("XML-RPC response error: {e:?}"))
        })?;
    let parsed: Wikitext = serde_json::from_str(&response.parsed)?;
    Ok(OutRoot {
        parsed,
        elapsed: response.elapsed,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_call_parser() {
        // TODO: Fix
        // let input = "{{PortGraph|name=TestGraph}}";
        // let result = call_parser(input, Config::default());
        // assert!(result.is_ok());
        // let output = result.unwrap();
        // assert!(!output.is_empty());
    }
}
