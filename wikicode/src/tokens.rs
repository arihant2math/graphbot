use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Text,
    TemplateOpen,
    TemplateParamSeparator,
    TemplateParamEquals,
    TemplateClose,
    ArgumentOpen,
    ArgumentSeparator,
    ArgumentClose,
    WikilinkOpen,
    WikilinkSeparator,
    WikilinkClose,
    ExternalLinkOpen,
    ExternalLinkSeparator,
    ExternalLinkClose,
    HTMLEntityStart,
    HTMLEntityNumeric,
    HTMLEntityHex,
    HTMLEntityEnd,
    HeadingStart,
    HeadingEnd,
    CommentStart,
    CommentEnd,
    TagOpenOpen,
    TagAttrStart,
    TagAttrEquals,
    TagAttrQuote,
    TagCloseOpen,
    TagCloseSelfclose,
    TagOpenClose,
    TagCloseClose,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Integer(i64),
    Bool(bool),
}

impl Value {
    pub fn string<S: Into<String>>(s: S) -> Self {
        Value::String(s.into())
    }

    pub fn integer(i: i64) -> Self {
        Value::Integer(i)
    }

    pub fn boolean(b: bool) -> Self {
        Value::Bool(b)
    }

    pub fn unwrap_string(self) -> String {
        match self {
            Value::String(s) => s,
            _ => panic!("Expected a string value, found {:?}", self),
        }
    }

    pub fn unwrap_integer(self) -> i64 {
        match self {
            Value::Integer(i) => i,
            _ => panic!("Expected an integer value, found {:?}", self),
        }
    }

    pub fn unwrap_bool(self) -> bool {
        match self {
            Value::Bool(b) => b,
            _ => panic!("Expected a boolean value, found {:?}", self),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub inner: HashMap<String, Value>,
    pub token_type: TokenType,
}

impl Deref for Token {
    type Target = HashMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Token {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

macro_rules! type_token {
    ($name:ident, $id:ident) => {
        impl Token {
            #[allow(dead_code, unused)]
            pub fn $name() -> Self {
                Self {
                    inner: HashMap::new(),
                    token_type: TokenType::$id,
                }
            }
        }
    };
}

type_token!(text, Text);
type_token!(template_open, TemplateOpen);
type_token!(template_param_separator, TemplateParamSeparator);
type_token!(template_param_equals, TemplateParamEquals);
type_token!(template_close, TemplateClose);
type_token!(argument_open, ArgumentOpen);
type_token!(argument_separator, ArgumentSeparator);
type_token!(argument_close, ArgumentClose);
type_token!(wikilink_open, WikilinkOpen);
type_token!(wikilink_separator, WikilinkSeparator);
type_token!(wikilink_close, WikilinkClose);
type_token!(external_link_open, ExternalLinkOpen);
type_token!(external_link_separator, ExternalLinkSeparator);
type_token!(external_link_close, ExternalLinkClose);
type_token!(html_entity_start, HTMLEntityStart);
type_token!(html_entity_numeric, HTMLEntityNumeric);
type_token!(html_entity_hex, HTMLEntityHex);
type_token!(html_entity_end, HTMLEntityEnd);
type_token!(heading_start, HeadingStart);
type_token!(heading_end, HeadingEnd);
type_token!(comment_start, CommentStart);
type_token!(comment_end, CommentEnd);
type_token!(tag_open_open, TagOpenOpen);
type_token!(tag_attr_start, TagAttrStart);
type_token!(tag_attr_equals, TagAttrEquals);
type_token!(tag_attr_quote, TagAttrQuote);
type_token!(tag_close_open, TagCloseOpen);
type_token!(tag_close_selfclose, TagCloseSelfclose);
type_token!(tag_open_close, TagOpenClose);
type_token!(tag_close_close, TagCloseClose);

// Text = make("Text")

// TemplateOpen = make("TemplateOpen")  # {{
// TemplateParamSeparator = make("TemplateParamSeparator")  # |
// TemplateParamEquals = make("TemplateParamEquals")  # =
// TemplateClose = make("TemplateClose")  # }}
//
// ArgumentOpen = make("ArgumentOpen")  # {{{
// ArgumentSeparator = make("ArgumentSeparator")  # |
// ArgumentClose = make("ArgumentClose")  # }}}
//
// WikilinkOpen = make("WikilinkOpen")  # [[
// WikilinkSeparator = make("WikilinkSeparator")  # |
// WikilinkClose = make("WikilinkClose")  # ]]
//
// ExternalLinkOpen = make("ExternalLinkOpen")  # [
// ExternalLinkSeparator = make("ExternalLinkSeparator")  #
// ExternalLinkClose = make("ExternalLinkClose")  # ]
//
// HTMLEntityStart = make("HTMLEntityStart")  # &
// HTMLEntityNumeric = make("HTMLEntityNumeric")  # #
// HTMLEntityHex = make("HTMLEntityHex")  # x
// HTMLEntityEnd = make("HTMLEntityEnd")  # ;
//
// HeadingStart = make("HeadingStart")  # =...
// HeadingEnd = make("HeadingEnd")  # =...
//
// CommentStart = make("CommentStart")  # <!--
// CommentEnd = make("CommentEnd")  # -->
//
// TagOpenOpen = make("TagOpenOpen")  # <
// TagAttrStart = make("TagAttrStart")
// TagAttrEquals = make("TagAttrEquals")  # =
// TagAttrQuote = make("TagAttrQuote")  # ", '
// TagCloseOpen = make("TagCloseOpen")  # >
// TagCloseSelfclose = make("TagCloseSelfclose")  # />
// TagOpenClose = make("TagOpenClose")  # </
// TagCloseClose = make("TagCloseClose")  # >
