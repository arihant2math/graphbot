use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::{BitOr, BitXor},
};

use either::Either;
use regex::Regex;

use crate::{
    contexts, definitions,
    definitions::get_html_tag,
    tokens::{Token, TokenType, Value},
};
use crate::tokenizer::TokenizerError::BadRoute;

fn split_with_captures<'t>(re: &Regex, text: &'t str) -> Vec<&'t str> {
    let mut pieces = Vec::new();
    let mut last_end = 0;

    for caps in re.captures_iter(text) {
        // caps.get(0) is the full match
        let mat = caps.get(0).unwrap();
        // 1) push the text before this match
        pieces.push(&text[last_end..mat.start()]);

        // 2) push each capturing‐group’s text (skip cap[0], which is the full match)
        for i in 1..caps.len() {
            if let Some(group) = caps.get(i) {
                pieces.push(group.as_str());
            }
        }

        last_end = mat.end();
    }

    // 3) finally push the trailing text after the last match
    pieces.push(&text[last_end..]);
    pieces
}

#[derive(Copy, Clone, Debug, thiserror::Error)]
pub enum TokenizerError {
    BadRoute(u64),
    UnexpectedTagCloseSelfClose,
    MissedTagCloseOpen,
    NonEmptyExitStack,
}

impl Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenizerError::BadRoute(context) => {
                write!(f, "Bad route encountered with context: {}", context)
            }
            TokenizerError::UnexpectedTagCloseSelfClose => {
                write!(f, "Unexpected tag close self-close")
            }
            TokenizerError::MissedTagCloseOpen => write!(f, "Missed tag close open"),
            TokenizerError::NonEmptyExitStack => write!(f, "Non-empty exit stack encountered"),
        }
    }
}

impl TokenizerError {
    fn unwrap_bad_route(self) -> u64 {
        match self {
            TokenizerError::BadRoute(context) => context,
            _ => panic!("Expected BadRoute, found {:?}", self),
        }
    }
}

bitflags::bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct TagOpenDataContext: u64 {
        const CX_NAME = 1 << 0;
        const CX_ATTR_READY = 1 << 1;
        const CX_ATTR_NAME = 1 << 2;
        const CX_ATTR_VALUE = 1 << 3;
        const CX_QUOTED = 1 << 4;
        const CX_NOTE_SPACE = 1 << 5;
        const CX_NOTE_EQUALS = 1 << 6;
        const CX_NOTE_QUOTE = 1 << 7;
    }
}

struct TagOpenData {
    pub context: TagOpenDataContext,
    pub padding_buffer: HashMap<String, String>,
    pub quoter: Option<String>,
    pub reset: i64,
}

impl TagOpenData {
    pub fn new() -> Self {
        let mut padding_buffer = HashMap::new();
        padding_buffer.insert("first".to_string(), String::new());
        padding_buffer.insert("before_eq".to_string(), String::new());
        padding_buffer.insert("after_eq".to_string(), String::new());
        TagOpenData {
            context: TagOpenDataContext::CX_NAME,
            padding_buffer,
            quoter: None,
            reset: 0,
        }
    }
}

#[derive(Debug)]
struct Stack {
    pub first: Vec<Token>,
    pub second: u64,
    pub third: Vec<String>,
    pub fourth: (i64, u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sentinel {
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker<'a> {
    Str(&'a str),
    Sentinel(Sentinel),
}

impl<'a> Marker<'a> {
    pub fn unwrap_str(self) -> &'a str {
        match self {
            Marker::Str(s) => s,
            Marker::Sentinel(_) => panic!("Expected a string marker, found a sentinel"),
        }
    }

    pub fn unwrap_sentinel(self) -> Sentinel {
        match self {
            Marker::Str(_) => panic!("Expected a sentinel marker, found a string"),
            Marker::Sentinel(s) => s,
        }
    }
}

impl From<Marker<'_>> for Either<String, Sentinel> {
    fn from(marker: Marker) -> Self {
        match marker {
            Marker::Str(s) => Either::Left(s.to_string()),
            Marker::Sentinel(s) => Either::Right(s),
        }
    }
}

pub const MARKERS: &[Marker<'static>] = &[
    Marker::Str("{"),
    Marker::Str("}"),
    Marker::Str("["),
    Marker::Str("]"),
    Marker::Str("<"),
    Marker::Str(">"),
    Marker::Str("|"),
    Marker::Str("="),
    Marker::Str("&"),
    Marker::Str("'"),
    Marker::Str("\""),
    Marker::Str("#"),
    Marker::Str("*"),
    Marker::Str(";"),
    Marker::Str(":"),
    Marker::Str("/"),
    Marker::Str("-"),
    Marker::Str("!"),
    Marker::Sentinel(Sentinel::Start),
    Marker::Sentinel(Sentinel::End),
];

const URISCHEME: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+.-";
const MAX_DEPTH: usize = 100;
pub struct Tokenizer {
    text: Vec<String>,
    head: i64,
    stacks: Vec<Stack>,
    global: u64,
    depth: usize,
    bad_routes: HashSet<(i64, u64)>,
    skip_style_tags: bool,
}
impl Tokenizer {
    fn regex() -> Regex {
        let pattern = r#"([{}\[\]<>|=&'#*;:/\\\"\-!\n])"#;
        regex::RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .unwrap()
    }

    pub fn new() -> Self {
        Tokenizer {
            text: Vec::new(),
            head: 0,
            stacks: Vec::new(),
            global: 0,
            depth: 0,
            bad_routes: HashSet::new(),
            skip_style_tags: false,
        }
    }

    fn stack(&self) -> &Vec<Token> {
        &self.stacks.last().unwrap().first
    }

    fn stack_mut(&mut self) -> &mut Vec<Token> {
        &mut self.stacks.last_mut().unwrap().first
    }

    fn context(&self) -> u64 {
        self.stacks.last().unwrap().second
    }

    fn context_mut(&mut self) -> &mut u64 {
        &mut self.stacks.last_mut().unwrap().second
    }

    fn textbuffer(&self) -> &Vec<String> {
        &self.stacks.last().unwrap().third
    }

    fn textbuffer_mut(&mut self) -> &mut Vec<String> {
        &mut self.stacks.last_mut().unwrap().third
    }

    fn set_textbuffer(&mut self, text: Vec<String>) {
        if let Some(stack) = self.stacks.last_mut() {
            stack.third = text;
        } else {
            unreachable!()
        }
    }

    fn stack_ident(&self) -> (i64, u64) {
        self.stacks.last().unwrap().fourth
    }

    /// Add a new token stack, context, and textbuffer to the list.
    fn push(&mut self, context: Option<u64>) -> Result<(), TokenizerError> {
        let context = context.unwrap_or(0);
        let new_ident = (self.head, context);
        if self.bad_routes.contains(&new_ident) {
            return Err(TokenizerError::BadRoute(context));
        }

        self.stacks.push(Stack {
            first: Vec::new(),
            second: context,
            third: Vec::new(),
            fourth: new_ident,
        });
        self.depth += 1;
        Ok(())
    }

    /// Push the textbuffer onto the stack as a Text node and clear it.
    fn push_textbuffer(&mut self) {
        if !self.textbuffer().is_empty() {
            let mut text_token = Token::text();
            text_token.insert(
                "text".to_string(),
                Value::String(self.textbuffer().join("")),
            );
            self.stack_mut().push(text_token);
            self.set_textbuffer(Vec::new());
        }
    }

    /// Pop the current stack/context/textbuffer, returning the stack.
    /// If *keep_context* is ``True``, then we will replace the underlying
    /// stack's context with the current stack's.
    fn pop(&mut self, keep_context: Option<bool>) -> Vec<Token> {
        let keep_context = keep_context.unwrap_or(false);
        self.push_textbuffer();
        self.depth -= 1;
        if keep_context {
            let context = self.context().clone();
            let stack = self.stacks.pop().unwrap().first;
            *self.context_mut() = context;
            return stack;
        }
        self.stacks.pop().unwrap().first
    }

    /// Return whether or not our max recursion depth has been exceeded.
    fn can_recurse(&self) -> bool {
        self.depth < MAX_DEPTH
    }

    /// Remember that the current route (head + context at push) is invalid.
    ///
    /// This will be noticed when calling _push with the same head and context,
    /// and the route will be failed immediately.
    fn memoize_bad_route(&mut self) {
        self.bad_routes.insert(self.stack_ident());
    }

    /// Fail the current tokenization route.
    /// Discards the current stack/context/textbuffer and raises BadRoute
    #[must_use]
    fn fail_route(&mut self) -> TokenizerError {
        let context = self.context().clone();
        self.memoize_bad_route();
        self.pop(None);
        TokenizerError::BadRoute(context)
    }

    /// Write a token to the end of the current token stack.
    fn emit(&mut self, token: Token) {
        self.push_textbuffer();
        self.stack_mut().push(token);
    }

    /// Write a token to the beginning of the current token stack.
    fn emit_first(&mut self, token: Token) {
        self.push_textbuffer();
        self.stack_mut().insert(0, token);
    }

    /// Write text to the current textbuffer.
    fn emit_text(&mut self, text: String) {
        self.textbuffer_mut().push(text);
    }

    /// Write a series of tokens to the current stack at once.
    fn emit_all(&mut self, tokenlist: Vec<Token>) {
        let mut tokenlist = tokenlist;
        if tokenlist.len() > 0 && matches!(tokenlist[0].token_type, TokenType::Text) {
            self.emit_text(
                tokenlist
                    .get(0)
                    .unwrap()
                    .get("text")
                    .unwrap()
                    .clone()
                    .unwrap_string(),
            );
            // TODO: O(n) performance hit here, maybe use VecDeque?
            tokenlist.remove(0);
        }
        self.push_textbuffer();
        self.stack_mut().extend(tokenlist);
    }

    fn emit_text_then_stack(&mut self, text: String) {
        let stack = self.pop(None);
        self.emit_text(text);
        if !stack.is_empty() {
            self.emit_all(stack);
        }
        self.head -= 1;
    }

    fn read(
        &mut self,
        delta: Option<i64>,
        strict: Option<bool>,
    ) -> Result<Either<String, Sentinel>, TokenizerError> {
        let delta = delta.unwrap_or(0);
        let strict = strict.unwrap_or(false);
        let index = self.head + delta;
        if index < 0 {
            return Ok(Either::Right(Sentinel::Start));
        }
        match self.text.get(index as usize).cloned() {
            Some(text) => Ok(Either::Left(text)),
            None => {
                if strict {
                    Err(self.fail_route())
                } else {
                    Ok(Either::Right(Sentinel::End))
                }
            }
        }
    }

    fn parse_template(&mut self, has_context: bool) -> Result<(), TokenizerError> {
        let reset = self.head.clone();
        let mut context = contexts::TEMPLATE_NAME;
        if has_context {
            context |= contexts::HAS_TEMPLATE;
        }
        match self.parse(Some(context), None) {
            Ok(template) => {
                self.emit_first(Token::template_open());
                self.emit_all(template);
                self.emit(Token::template_close());
                Ok(())
            }
            Err(e) => {
                self.head = reset;
                Err(e)
            }
        }
    }

    fn parse_argument(&mut self) -> Result<(), TokenizerError> {
        let reset = self.head.clone();
        match self.parse(Some(contexts::ARGUMENT_NAME), None) {
            Ok(argument) => {
                self.emit_first(Token::argument_open());
                self.emit_all(argument);
                self.emit(Token::argument_close());
                Ok(())
            }
            Err(e) => {
                self.head = reset;
                Err(e)
            }
        }
    }

    fn parse_template_or_argument(&mut self) -> Result<(), TokenizerError> {
        self.head += 2;
        let mut braces = 2;
        while self.read(None, None)? == Either::Left("{".to_string()) {
            self.head += 1;
            braces += 1;
        }
        let mut has_content = false;
        self.push(None)?;

        while braces > 0 {
            if braces == 1 {
                self.emit_text_then_stack("{".to_string());
                return Ok(());
            }
            if braces == 2 {
                match self.parse_template(has_content) {
                    Err(TokenizerError::BadRoute(_)) => {
                        self.emit_text_then_stack("{{".to_string());
                    }
                    Err(e) => {
                        return Err(e);
                    }
                    Ok(_) => {}
                }
                break;
            }
            match self.parse_argument() {
                Ok(_) => {
                    braces -= 2;
                }
                Err(_) => match self.parse_template(has_content) {
                    Ok(_) => {
                        braces -= 2;
                    }
                    Err(_) => self.emit_text_then_stack("{".repeat(braces as usize)),
                },
            }
            if braces != 0 {
                has_content = true;
                self.head += 1;
            }
        }

        let tmp = self.pop(None);
        self.emit_all(tmp);
        if (self.context() & contexts::FAIL_NEXT) != 0 {
            *self.context_mut() ^= contexts::FAIL_NEXT;
        }
        Ok(())
    }

    fn handle_template_param(&mut self) -> Result<(), TokenizerError> {
        if self.context() & contexts::TEMPLATE_NAME != 0 {
            if (self.context() & (contexts::HAS_TEXT | contexts::HAS_TEMPLATE)) == 0 {
                return Err(self.fail_route());
            }
            *self.context_mut() ^= contexts::TEMPLATE_NAME;
        } else if self.context() & contexts::TEMPLATE_PARAM_VALUE != 0 {
            *self.context_mut() ^= contexts::TEMPLATE_PARAM_VALUE;
        } else {
            let tmp = self.pop(None);
            self.emit_all(tmp);
        }
        *self.context_mut() |= contexts::TEMPLATE_PARAM_KEY;
        self.emit(Token::template_param_separator());
        self.push(Some(self.context()))?;
        Ok(())
    }

    fn handle_template_param_value(&mut self) -> Result<(), TokenizerError> {
        let tmp = self.pop(None);
        self.emit_all(tmp);
        *self.context_mut() ^= contexts::TEMPLATE_PARAM_KEY;
        *self.context_mut() |= contexts::TEMPLATE_PARAM_VALUE;
        self.emit(Token::template_param_equals());
        Ok(())
    }

    /// Handle the end of a template at the head of the string.
    fn handle_template_end(&mut self) -> Result<Vec<Token>, TokenizerError> {
        if self.context() & contexts::TEMPLATE_NAME != 0 {
            if (self.context() & (contexts::HAS_TEXT | contexts::HAS_TEMPLATE)) == 0 {
                return Err(self.fail_route());
            }
        } else if self.context() & contexts::TEMPLATE_PARAM_KEY != 0 {
            let tmp = self.pop(None);
            self.emit_all(tmp);
        }
        self.head += 1;
        Ok(self.pop(None))
    }

    /// Handle the separator between an argument's name and default.
    fn handle_argument_separator(&mut self) -> Result<(), TokenizerError> {
        *self.context_mut() ^= contexts::ARGUMENT_NAME;
        *self.context_mut() |= contexts::ARGUMENT_DEFAULT;
        self.emit(Token::argument_separator());
        Ok(())
    }

    /// Handle the end of an argument at the head of the string.
    fn handle_argument_end(&mut self) -> Result<Vec<Token>, TokenizerError> {
        self.head += 2;
        Ok(self.pop(None))
    }

    /// Parse an internal wikilink at the head of the wikicode string.
    fn parse_wikilink(&mut self) -> Result<(), TokenizerError> {
        let reset = self.head + 1;
        self.head += 2;
        match self.really_parse_external_link(true) {
            Ok((link, _extra)) => {
                if self.context() & contexts::EXT_LINK_TITLE != 0 {
                    self.head = reset;
                    self.emit_text("[[".to_string());
                    return Ok(());
                }
                self.emit_text("[".to_string());
                let mut tmp = Token::external_link_open();
                tmp.insert("brackets".to_string(), Value::Bool(true));
                self.emit(tmp);
                self.emit_all(link);
                self.emit(Token::external_link_close());
                Ok(())
            }
            Err(_) => {
                self.head = reset + 1;
                match self.parse(Some(contexts::WIKILINK_TITLE), None) {
                    Ok(wikilink) => {
                        self.emit(Token::wikilink_open());
                        self.emit_all(wikilink);
                        self.emit(Token::wikilink_close());
                    }
                    Err(e) => {
                        self.head = reset;
                        self.emit_text("[[".to_string());
                    }
                }
                return Ok(());
            }
        }
    }

    fn handle_wikilink_separator(&mut self) {
        *self.context_mut() ^= contexts::WIKILINK_TITLE;
        *self.context_mut() |= contexts::WIKILINK_TEXT;
        self.emit(Token::wikilink_separator());
    }

    fn handle_wikilink_end(&mut self) -> Vec<Token> {
        self.head += 1;
        self.pop(None)
    }

    fn parse_bracketed_uri_scheme(&mut self) -> Result<(), TokenizerError> {
        self.push(Some(contexts::EXT_LINK_URI as _))?;
        if self.read(None, None)?.unwrap_left() == self.read(Some(1), None)?.unwrap_left()
            && self.read(None, None)?.unwrap_left() == "/"
        {
            self.emit_text("//".to_string());
            self.head += 2;
        } else {
            fn all_valid(this: &str) -> bool {
                this.chars().all(|c| URISCHEME.contains(c))
            }

            let mut scheme = String::new();
            while let Either::Left(this) = self.read(None, None)? {
                if !all_valid(&this) {
                    break;
                }
                scheme.push_str(&this);
                self.emit_text(this);
                self.head += 1;
            }

            if self.read(None, None)? != Either::Left(":".to_string()) {
                return Err(self.fail_route());
            }
            self.emit_text(":".to_string());
            self.head += 1;

            let slashes = self.read(None, None)?.unwrap_left()
                == self.read(Some(1), None)?.unwrap_left()
                && self.read(None, None)?.unwrap_left() == "/";
            if slashes {
                self.emit_text("//".to_string());
                self.head += 2;
            }

            if !definitions::is_scheme(&scheme, slashes) {
                return Err(self.fail_route());
            }
        }
        Ok(())
    }

    fn parse_free_uri_scheme(&mut self) -> Result<(), TokenizerError> {
        let mut scheme = String::new();
        let mut backtrack = self.textbuffer().clone();

        while let Some(chunk) = backtrack.pop() {
            for char in chunk.chars().rev() {
                if !char.is_alphanumeric() && !['+', '-', '.'].contains(&char) {
                    break;
                }
                if !URISCHEME.contains(char) {
                    return Err(self.fail_route());
                }
                scheme.insert(0, char);
            }
        }

        let slashes = self.read(None, None)?.unwrap_left()
            == self.read(Some(1), None)?.unwrap_left()
            && self.read(None, None)?.unwrap_left() == "/";
        if !definitions::is_scheme(&scheme, slashes) {
            return Err(self.fail_route());
        }

        *self.context_mut() |= contexts::EXT_LINK_URI;
        self.emit_text(scheme);
        self.emit_text(":".to_string());
        if slashes {
            self.emit_text("//".to_string());
            self.head += 2;
        }

        Ok(())
    }

    /// Handle text in a free external link, including trailing punctuation.
    fn handle_free_link_text(
        &mut self,
        punct: &str,
        mut tail: String,
        this: &str,
    ) -> (String, String) {
        let mut punct = punct.to_string();
        if this.contains('(') && punct.contains(')') {
            punct.pop(); // ')' is no longer valid punctuation
        }
        if this.ends_with(&punct) {
            let mut i = this.len();
            for (index, char) in this.chars().rev().enumerate() {
                if !punct.contains(char) {
                    i = this.len() - index - 1;
                    break;
                }
            }
            let stripped = &this[..i];
            if !stripped.is_empty() && !tail.is_empty() {
                self.emit_text(tail.clone());
                tail.clear();
            }
            tail.push_str(&this[i..]);
            self.emit_text(stripped.to_string());
        } else if !tail.is_empty() {
            self.emit_text(tail.clone());
            tail.clear();
        }
        self.emit_text(this.to_string());
        (punct, tail)
    }

    /// Return whether the current head is the end of a URI.
    fn is_uri_end(&mut self, this: &str, nxt: &str) -> bool {
        let after = self
            .read(Some(2), None)
            .unwrap_or(Either::Right(Sentinel::End));
        let ctx = self.context();
        this == "\n"
            || this == "["
            || this == "]"
            || this == "<"
            || this == ">"
            || this == "\""
            || this.contains(' ')
            || (this == nxt && this == "'")
            || (this == "|" && (ctx & contexts::TEMPLATE != 0))
            || (this == "=" && (ctx & (contexts::TEMPLATE_PARAM_KEY | contexts::HEADING) != 0))
            || (this == nxt && this == "}" && (ctx & contexts::TEMPLATE != 0))
            || (this == nxt
                && this == "}"
                && after == Either::Left("}".to_string())
                && (ctx & contexts::ARGUMENT != 0))
    }

    fn really_parse_external_link(
        &mut self,
        brackets: bool,
    ) -> Result<(Vec<Token>, Option<String>), TokenizerError> {
        let (invalid, mut punct) = if brackets {
            self.parse_bracketed_uri_scheme()?;
            (vec!["\n", " ", "]"], Vec::new())
        } else {
            self.parse_free_uri_scheme()?;
            (
                vec!["\n", " ", "[", "]"],
                vec![',', ';', '\\', '.', ':', '!', '?', ')'],
            )
        };

        if let Either::Left(this) = self.read(None, None)? {
            if this == "END" || invalid.contains(&this.as_str()) {
                return Err(self.fail_route());
            }
        }

        let mut tail = String::new();
        loop {
            let this = self.read(None, None)?;
            let nxt = self.read(Some(1), None)?;

            match this {
                Either::Left(ref s) if s == "&" => {
                    if !tail.is_empty() {
                        self.emit_text(tail.clone());
                        tail.clear();
                    }
                    self.parse_entity()?;
                }
                Either::Left(ref s) if s == "<" && nxt == Either::Left("!".to_string()) => {
                    if self.read(Some(2), None)? == self.read(Some(3), None)?
                        && self.read(Some(2), None)? == Either::Left("-".to_string())
                    {
                        if !tail.is_empty() {
                            self.emit_text(tail.clone());
                            tail.clear();
                        }
                        self.parse_comment()?;
                    }
                }
                Either::Left(ref s)
                    if s == "{" && nxt == Either::Left("{".to_string()) && self.can_recurse() =>
                {
                    if !tail.is_empty() {
                        self.emit_text(tail.clone());
                        tail.clear();
                    }
                    self.parse_template_or_argument()?;
                }
                _ if brackets => {
                    if this == Either::Right(Sentinel::End)
                        || this == Either::Left("\n".to_string())
                    {
                        return Err(self.fail_route());
                    }
                    if this == Either::Left("]".to_string()) {
                        return Ok((self.pop(None), None));
                    }
                    if self.is_uri_end(&this.clone().unwrap_left(), &nxt.unwrap_left()) {
                        if this.clone().unwrap_left().contains(' ') {
                            let parts: Vec<String> = this
                                .unwrap_left()
                                .splitn(2, ' ')
                                .map(str::to_string)
                                .collect();
                            self.emit_text(parts[0].to_string());
                            self.emit(Token::external_link_separator());
                            if parts.len() > 1 {
                                self.emit_text(parts[1].to_string());
                            }
                            self.head += 1;
                        } else {
                            let mut separator = Token::external_link_separator();
                            separator.insert("suppress_space".to_string(), Value::boolean(true));
                            self.emit(separator);
                        }
                        *self.context_mut() ^= contexts::EXT_LINK_URI;
                        *self.context_mut() |= contexts::EXT_LINK_TITLE;
                        return Ok((self.parse(None, Some(false))?, None));
                    }
                    self.emit_text(this.unwrap_left());
                }
                _ => {
                    if self.is_uri_end(&this.clone().unwrap_left(), &nxt.unwrap_left()) {
                        if this != Either::Right(Sentinel::End)
                            && this.clone().unwrap_left().contains(' ')
                        {
                            let parts: Vec<String> = this
                                .unwrap_left()
                                .splitn(2, ' ')
                                .map(|s| s.to_string())
                                .collect();
                            let (new_punct, new_tail) = self.handle_free_link_text(
                                &punct.iter().collect::<String>(),
                                tail.clone(),
                                &parts[0],
                            );
                            punct = new_punct.chars().collect();
                            tail = new_tail + " " + &parts[1];
                        } else {
                            self.head -= 1;
                        }
                        return Ok((self.pop(None), Some(tail)));
                    }
                    let (new_punct, new_tail) = self.handle_free_link_text(
                        &punct.iter().collect::<String>(),
                        tail.clone(),
                        &this.unwrap_left(),
                    );
                    punct = new_punct.chars().collect();
                    tail = new_tail;
                }
            }
            self.head += 1;
        }
    }

    fn remove_uri_scheme_from_textbuffer(&mut self, scheme: &str) {
        let mut length = scheme.len();
        while length > 0 {
            if let Some(last) = self.textbuffer_mut().last_mut() {
                if length < last.len() {
                    *last = last[..last.len() - length].to_string();
                    break;
                }
                length -= last.len();
                self.textbuffer_mut().pop();
            } else {
                break;
            }
        }
    }

    fn parse_external_link(&mut self, brackets: bool) -> Result<(), TokenizerError> {
        if (self.context() & contexts::NO_EXT_LINKS != 0) || !self.can_recurse() {
            if !brackets && (self.context() & contexts::DL_TERM != 0) {
                self.handle_dl_term()?;
            } else {
                let tmp = self.read(None, None)?.unwrap_left();
                self.emit_text(tmp);
            }
            return Ok(());
        }

        let reset = self.head;
        self.head += 1;

        match self.really_parse_external_link(brackets) {
            Ok((link, extra)) => {
                if !brackets {
                    let tmp = link[0].get("text").unwrap().clone().unwrap_string();
                    let scheme = tmp.split(':').next().unwrap().to_string();
                    self.remove_uri_scheme_from_textbuffer(&scheme);
                }
                let mut tmp = Token::external_link_open();
                tmp.insert("brackets".to_string(), Value::Bool(brackets));
                self.emit(tmp);
                self.emit_all(link);
                self.emit(Token::external_link_close());
                if let Some(extra_text) = extra {
                    self.emit_text(extra_text);
                }
                Ok(())
            }
            Err(_) => {
                self.head = reset;
                if !brackets && (self.context() & contexts::DL_TERM != 0) {
                    self.handle_dl_term()?;
                } else {
                    let tmp = self.read(None, None)?.unwrap_left();
                    self.emit_text(tmp);
                }
                Ok(())
            }
        }
    }

    fn parse_heading(&mut self) -> Result<(), TokenizerError> {
        self.global |= contexts::GL_HEADING;
        let reset = self.head.clone();
        self.head += 1;
        let mut best = 1;
        while self.read(None, None)? == Either::Left("=".to_string()) {
            best += 1;
            self.head += 1;
        }
        let context = contexts::HEADING_LEVEL_1 << (best - 1).min(5);

        match self.parse(Some(context), None) {
            Ok(after) => {
                // IMPORTANT: THIS IS A HACK TO TRY TO KEEP THE ARCHITECTURE IN LINE WITH PYTHON
                let mut after = after;
                let last = after.pop().unwrap();
                // Now follow python
                let title = after;
                let level = last.get("level").cloned().unwrap().unwrap_integer() as usize;
                self.emit(last);
                if level < best {
                    self.emit_text("=".repeat(best as usize - level));
                }
                self.emit_all(title);
                self.emit(Token::heading_end());
            }
            Err(_) => {
                self.head = reset + (best as i64) - 1;
                self.emit_text("=".repeat(best));
            }
        }
        self.global ^= contexts::GL_HEADING;
        Ok(())
    }

    /// Handle the end of a section heading at the head of the string.
    fn handle_heading_end(&mut self) -> Result<(Vec<Token>, usize), TokenizerError> {
        let reset = self.head;
        self.head += 1;
        let mut best: i64 = 1;

        while self.read(None, None)? == Either::Left("=".to_string()) {
            best += 1;
            self.head += 1;
        }

        let current = ((self.context() / contexts::HEADING_LEVEL_1) as f64)
            .log2()
            .ceil() as usize
            + 1;
        let level = current.min(best.min(6) as usize);

        match self.parse(Some(self.context()), None) {
            Ok(after) => {
                self.emit_text("=".repeat(best as usize));
                self.emit_all(after);
                Ok((self.pop(None), level))
            }
            Err(BadRoute(_)) => {
                if level < best as usize {
                    self.emit_text("=".repeat(best as usize - level));
                }
                self.head = reset + best - 1;
                Ok((self.pop(None), level))
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn really_parse_entity(&mut self) -> Result<(), TokenizerError> {
        self.emit(Token::html_entity_start());
        self.head += 1;

        let mut this = self.read(None, Some(true))?.unwrap_left();
        let (numeric, hexadecimal) = if &this == "#" {
            self.emit(Token::html_entity_numeric());
            self.head += 1;
            this = self.read(None, Some(true))?.unwrap_left();
            if this.chars().next().unwrap() == 'x' {
                let mut tmp = Token::html_entity_hex();
                tmp.insert(
                    "char".to_lowercase(),
                    Value::String(this.chars().next().unwrap().to_string()),
                );
                self.emit(tmp);
                this = this[1..].to_string();
                if this.is_empty() {
                    return Err(self.fail_route());
                }
                (true, true)
            } else {
                (true, false)
            }
        } else {
            (false, false)
        };

        let mut valid = if hexadecimal {
            "0123456789abcdefABCDEF"
        } else {
            "0123456789"
        }
        .to_string();
        if !numeric && !hexadecimal {
            valid += "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
        }
        if this.chars().into_iter().any(|c| !valid.contains(c)) {
            return Err(self.fail_route());
        }

        self.head += 1;
        if self.read(None, None)? != Either::Left(";".to_string()) {
            return Err(self.fail_route());
        }
        if numeric {
            let test = if hexadecimal {
                i64::from_str_radix(&this, 16)
            } else {
                i64::from_str_radix(&this, 10)
            }
            .unwrap();
            if test < 1 || test > 0x10FFFF {
                return Err(self.fail_route());
            }
        } else {
            // TODO: not correct
            // if this not in html.entities.entitydefs:
            //     self._fail_route()
            return Err(self.fail_route());
        }

        let mut text = Token::text();
        text.insert("text".to_string(), Value::String(this));
        self.emit(text);
        self.emit(Token::html_entity_end());
        Ok(())
    }

    fn parse_entity(&mut self) -> Result<(), TokenizerError> {
        let reset = self.head;
        self.push(Some(contexts::HTML_ENTITY))?;
        match self.really_parse_entity() {
            Ok(()) => {
                let tmp = self.pop(None);
                self.emit_all(tmp);
            }
            Err(TokenizerError::BadRoute(_)) => {
                self.head = reset;
                let text = self.read(None, None)?.unwrap_left();
                self.emit_text(text);
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(())
    }

    /// Parse an HTML comment at the head of the wikicode string.
    fn parse_comment(&mut self) -> Result<(), TokenizerError> {
        self.head += 4;
        let reset = self.head - 1;
        self.push(None)?;
        loop {
            let this = self.read(None, None)?;
            if this == Either::Right(Sentinel::End) {
                self.pop(None);
                self.head = reset;
                self.emit_text("<!--".to_string());
                return Ok(());
            }
            if this == self.read(Some(1), None)?
                && this == Either::Left("-".to_string())
                && self.read(Some(2), None)? == Either::Left(">".to_string())
            {
                self.emit_first(Token::comment_start());
                self.emit(Token::comment_end());
                let tmp = self.pop(None);
                self.emit_all(tmp);
                self.head += 2;
                if self.context() & contexts::FAIL_NEXT != 0 {
                    // verify_safe() sets this flag while parsing a template
                    // or link when it encounters what might be a comment -- we
                    // must unset it to let verify_safe() know it was correct:
                    *self.context_mut() ^= contexts::FAIL_NEXT;
                }
                return Ok(());
            }
            self.emit_text(this.unwrap_left());
            self.head += 1;
        }
    }

    // TODO: push_tag_buffer()
    fn push_tag_buffer(&mut self, data: &mut TagOpenData) {
        if data.context & TagOpenDataContext::CX_QUOTED != 0 {
            let mut tmp = Token::tag_attr_quote();
            tmp.insert("char".to_string(), Value::String(data.quoter.unwrap()));
            self.emit_first(tmp);
            let tmp = self.pop(None);
            self.emit_all(tmp);
        }
        let buf = data.padding_buffer;
        let mut tmp = Token::tag_attr_start();
        tmp.insert("pad_first".to_string(), Value::String(buf.get("first").unwrap().to_string()));
        tmp.insert("pad_before_eq".to_string(), Value::String(buf.get("before_eq").unwrap().to_string()));
        tmp.insert("pad_after_eq".to_string(), Value::String(buf.get("after_eq").unwrap().to_string()));
        self.emit_first(tmp);
        let tmp = self.pop(None);
        self.emit_all(tmp);
        data.padding_buffer.iter_mut().for_each(|(k, v)| {
            v.clear();
        });
    }

    // TODO: handle_tag_space()
    fn handle_tag_text(&mut self, text: String) -> Result<(), TokenizerError> {
        let nxt = self.read(Some(1), None)?;
        if !self.can_recurse() || !MARKERS.contains(&Marker::Str(&text)) {
            self.emit_text(text);
        } else if Either::Left(text.clone()) == nxt && nxt == Either::Left("{".to_string()) {
            self.parse_template_or_argument()?;
        } else if Either::Left(text.clone()) == nxt && nxt == Either::Left("[".to_string()) {
            self.parse_wikilink()?;
        } else if Either::Left(text.clone()) == nxt && nxt == Either::Left("<".to_string()) {
            self.parse_tag()?;
        } else {
            self.emit_text(text);
        }
        Ok(())
    }

    // TODO: handle_tag_data()
    // TODO: handle_tag_close_open()
    fn handle_tag_open_close(&mut self) -> Result<(), TokenizerError> {
        self.emit(Token::tag_open_close());
        self.push(Some(contexts::TAG_CLOSE))?;
        self.head += 1;
        Ok(())
    }

    fn handle_tag_close_close(&mut self) -> Result<Vec<Token>, TokenizerError> {
        fn strip(token: Token) -> String {
            let text = token.get("text").cloned().unwrap().unwrap_string();
            text.trim_end().to_lowercase()
        }

        let closing = self.pop(None);
        if closing.len() != 1
            || (!matches!(closing[0].token_type.clone(), TokenType::Text)
                || (strip(closing[0].clone()) != strip(self.stack()[1].clone())))
        {
            return Err(self.fail_route());
        }
        self.emit_all(closing);
        self.emit(Token::tag_close_close());
        return Ok(self.pop(None));
    }

    // TODO: handle_blacklisted_tag()
    fn handle_single_only_tag_end(&mut self) -> Vec<Token> {
        let padding = self
            .stack_mut()
            .pop()
            .unwrap()
            .get("padding")
            .cloned()
            .unwrap();
        let mut tmp = Token::tag_close_selfclose();
        tmp.insert("padding".to_string(), padding);
        self.emit(tmp);
        self.head -= 1;
        return self.pop(None);
    }

    fn handle_single_tag_end(&mut self) -> Result<Vec<Token>, TokenizerError> {
        let stack = self.stack_mut();
        // We need to find the index of the TagCloseOpen token corresponding to
        // the TagOpenOpen token located at index 0:
        let mut depth = 1;
        let mut index = 2;

        while index < stack.len() {
            match stack[index].token_type {
                TokenType::TagOpenOpen => {
                    depth += 1;
                }
                TokenType::TagCloseOpen => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                TokenType::TagCloseSelfclose => {
                    depth -= 1;
                    if depth == 0 {
                        return Err(TokenizerError::UnexpectedTagCloseSelfClose);
                    }
                }
                _ => {}
            }
            index += 1;
        }

        if depth != 0 {
            return Err(TokenizerError::MissedTagCloseOpen);
        }

        let padding = stack[index]
            .get("padding")
            .cloned()
            .unwrap_or(Value::String("".to_string()));
        let mut tmp = Token::tag_close_selfclose();
        tmp.insert("padding".to_string(), padding);
        stack[index] = tmp;
        Ok(self.pop(None))
    }

    fn really_parse_tag(&mut self) -> Result<Vec<Token>, TokenizerError> {
        let mut data = TagOpenData::new();
        self.push(Some(contexts::TAG_OPEN))?;
        self.emit(Token::tag_open_open());
        loop {
            let (this, nxt) = (self.read(None, None)?, self.read(Some(1), None)?);
            let can_exit = data
                .context
                .bitxor(TagOpenDataContext::CX_QUOTED.bitor(TagOpenDataContext::CX_NAME))
                .bits()
                == 0
                || data
                    .context
                    .bitxor(TagOpenDataContext::CX_NOTE_SPACE)
                    .bits()
                    != 0;
            if this == Either::Right(Sentinel::End) {
                if self.context() & contexts::TAG_ATTR != 0 {
                    if data.context.bitxor(TagOpenDataContext::CX_QUOTED).bits() != 0 {
                        data.context = TagOpenDataContext::CX_ATTR_VALUE;
                        self.memoize_bad_route();
                        self.pop(None);
                        self.head = data.reset;
                        continue;
                    }
                    self.pop(None);
                }
                return Err(self.fail_route());
            } else if this == Either::Left(">".to_string()) && can_exit {
                todo!();
            }
            todo!();
        }
        // TODO: Fix
        todo!();
    }

    fn handle_invalid_tag_start(&mut self) -> Result<(), TokenizerError> {
        let reset = self.head.clone() + 1;
        self.head += 2;
        assert_ne!(self.read(None, None)?, Either::Right(Sentinel::End));
        // TODO: check for is_single_only
        let mut tag = self.really_parse_tag()?;
        tag[0].insert("invalid".to_string(), Value::Bool(true));
        self.emit_all(tag);
        Ok(())
    }

    fn parse_tag(&mut self) -> Result<(), TokenizerError> {
        let reset = self.head.clone();
        self.head += 1;
        match self.really_parse_tag() {
            Ok(tag) => self.emit_all(tag),
            Err(_) => {
                self.head = reset;
                self.emit_text("<".to_string());
            }
        }
        Ok(())
    }

    fn emit_style_tag(&mut self, tag: String, markup: String, body: Vec<Token>) {
        let mut tmp = Token::tag_open_open();
        tmp.insert("wiki_markup".to_string(), Value::String(markup));
        self.emit(tmp);
        self.emit_text(tag.clone());
        self.emit(Token::tag_close_open());
        self.emit_all(body);
        self.emit(Token::tag_open_close());
        self.emit_text(tag);
        self.emit(Token::tag_close_close());
    }

    fn parse_italics(&mut self) -> Result<(), TokenizerError> {
        let reset = self.head;
        match self.parse(Some(contexts::STYLE_ITALICS), None) {
            Ok(stack) => self.emit_style_tag("i".to_string(), "''".to_string(), stack),
            Err(route) => {
                self.head = reset;
                if route.unwrap_bad_route() & contexts::STYLE_PASS_AGAIN != 0 {
                    let new_ctx = contexts::STYLE_ITALICS | contexts::STYLE_SECOND_PASS;
                    match self.parse(Some(new_ctx), None) {
                        Ok(stack) => self.emit_style_tag("i".to_string(), "''".to_string(), stack),
                        Err(_) => {
                            self.head = reset;
                            self.emit_text("''".to_string());
                        }
                    }
                } else {
                    self.emit_text("''".to_string());
                }
            }
        }
        Ok(())
    }

    fn parse_bold(&mut self) -> Result<bool, TokenizerError> {
        let reset = self.head;
        match self.parse(Some(contexts::STYLE_BOLD), None) {
            Ok(stack) => {
                self.emit_style_tag("b".to_string(), "'''".to_string(), stack);
                Ok(false)
            }
            Err(route) => {
                self.head = reset;
                if self.context() & contexts::STYLE_SECOND_PASS != 0 {
                    self.emit_text("'".to_string());
                    return Ok(true);
                }
                if self.context() & contexts::STYLE_ITALICS != 0 {
                    *self.context_mut() |= contexts::STYLE_PASS_AGAIN;
                    self.emit_text("'''".to_string());
                } else {
                    self.emit_text("'".to_string());
                    self.parse_italics()?;
                }
                Ok(false)
            }
        }
    }

    fn parse_italics_and_bold(&mut self) -> Result<(), TokenizerError> {
        let reset = self.head;
        match self.parse(Some(contexts::STYLE_BOLD), None) {
            Ok(stack) => {
                let reset = self.head;
                match self.parse(Some(contexts::STYLE_ITALICS), None) {
                    Ok(stack2) => {
                        self.push(None)?;
                        self.emit_style_tag("b".to_string(), "'''".to_string(), stack);
                        self.emit_all(stack2);
                        let tmp = self.pop(None);
                        self.emit_style_tag("i".to_string(), "''".to_string(), tmp);
                    }
                    Err(_) => {
                        self.head = reset;
                        self.emit_text("''".to_string());
                        self.emit_style_tag("b".to_string(), "'''".to_string(), stack);
                    }
                }
            }
            Err(_) => {
                self.head = reset;
                match self.parse(Some(contexts::STYLE_ITALICS), None) {
                    Ok(stack) => {
                        let reset = self.head;
                        match self.parse(Some(contexts::STYLE_BOLD), None) {
                            Ok(stack2) => {
                                self.push(None)?;
                                self.emit_style_tag("i".to_string(), "''".to_string(), stack);
                                self.emit_all(stack2);
                                let tmp = self.pop(None);
                                self.emit_style_tag("b".to_string(), "'''".to_string(), tmp);
                            }
                            Err(_) => {
                                self.head = reset;
                                self.emit_text("'''".to_string());
                                self.emit_style_tag("i".to_string(), "''".to_string(), stack);
                            }
                        }
                    }
                    Err(_) => {
                        self.head = reset;
                        self.emit_text("'''''".to_string());
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_style(&mut self) -> Result<Option<Vec<Token>>, TokenizerError> {
        self.head += 2;
        let mut ticks = 2;
        while self.read(None, None)? == Either::Left("'".to_string()) {
            self.head += 1;
            ticks += 1;
        }
        let italics = self.context() & contexts::STYLE_ITALICS != 0;
        let bold = self.context() & contexts::STYLE_BOLD != 0;

        if ticks > 5 {
            self.emit_text("'".repeat(ticks - 5));
            ticks = 5;
        } else if ticks == 4 {
            self.emit_text("'".to_string());
            ticks = 3;
        }

        if (italics && (ticks == 2 || ticks == 5)) || (bold && (ticks == 3 || ticks == 5)) {
            if ticks == 5 {
                self.head -= if italics { 3 } else { 2 };
            }
            return Ok(Some(self.pop(None)));
        }
        if !self.can_recurse() {
            if ticks == 3 {
                if self.context() & contexts::STYLE_SECOND_PASS != 0 {
                    self.emit_text("'".to_string());
                    return Ok(Some(self.pop(None)));
                }
                if self.context() & contexts::STYLE_ITALICS != 0 {
                    *self.context_mut() |= contexts::STYLE_PASS_AGAIN;
                }
            }
            self.emit_text("'".repeat(ticks));
        } else if ticks == 2 {
            self.parse_italics()?;
        } else if ticks == 3 {
            if self.parse_bold()? {
                return Ok(Some(self.pop(None)));
            }
        } else {
            self.parse_italics_and_bold()?;
        }
        self.head -= 1;
        Ok(None)
    }

    /// Handle a list marker at the head (``#``, ``*``, ``;``, ``:``).
    fn handle_list_marker(&mut self) -> Result<(), TokenizerError> {
        let markup = self.read(None, None)?;
        assert_ne!(markup, Either::Right(Sentinel::End));
        if markup == Either::Left(";".to_string()) {
            *self.context_mut() |= contexts::DL_TERM
        }
        let mut tmp = Token::tag_open_open();
        tmp.insert(
            "wiki_markup".to_string(),
            Value::String(markup.clone().unwrap_left()),
        );
        self.emit_text(get_html_tag(&markup.unwrap_left()).to_string());
        self.emit(Token::tag_close_selfclose());
        Ok(())
    }

    /// Handle a wiki-style list (``#``, ``*``, ``;``, ``:``).
    fn handle_list(&mut self) -> Result<(), TokenizerError> {
        self.handle_list_marker()?;
        let matchers = [
            "#".to_string(),
            "*".to_string(),
            ";".to_string(),
            ":".to_string(),
        ];
        while matchers.contains(&self.read(Some(1), None)?.unwrap_left()) {
            self.head += 1;
            self.handle_list_marker()?;
        }
        Ok(())
    }

    /// Handle a wiki-style horizontal rule (``----``) in the string.
    fn handle_hr(&mut self) -> Result<(), TokenizerError> {
        let mut length = 4;
        self.head += 3;
        while self.read(Some(1), None)? == Either::Left("-".to_string()) {
            length += 1;
            self.head += 1;
        }
        let mut tmp = Token::tag_open_open();
        tmp.insert(
            "wiki_markup".to_string(),
            Value::String("-".repeat(length as usize)),
        );
        self.emit(tmp);
        self.emit_text("hr".to_string());
        self.emit(Token::tag_close_selfclose());
        Ok(())
    }

    /// Handle the term in a description list (``foo`` in ``;foo:bar``).
    fn handle_dl_term(&mut self) -> Result<(), TokenizerError> {
        *self.context_mut() ^= contexts::DL_TERM;
        if self.read(None, None)? == Either::Left(":".to_string()) {
            self.handle_list_marker()?;
        } else {
            self.emit_text("\n".to_string());
        }
        Ok(())
    }

    // TOOD: emit_table_tag
    fn handle_table_style(&mut self, end_token: &str) -> Result<(), TokenizerError> {
        // TODO: Finish
        todo!();
    }

    fn parse_table(&mut self) -> Result<(), TokenizerError> {
        let reset = self.head;
        self.head += 2;
        let padding = match self.push(Some(contexts::TABLE_OPEN)) {
            Ok(_) => self.handle_table_style("\n")?,
            Err(BadRoute(_)) => {
                self.head = reset;
                self.emit_text("{|".to_string());
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        };

        self.head += 1;
        let restore_point = self.stack_ident();
        // TODO: Finish
        todo!()
    }

    fn handle_table_row(&mut self) {
        // TODO: Finish
        todo!()
    }

    fn handle_table_cell(
        &mut self,
        markup: String,
        tag: String,
        line_context: u64,
    ) -> Result<(), TokenizerError> {
        let old_context = self.context();
        let mut padding = String::new();
        let mut style = Vec::new();
        self.head += markup.len() as i64;
        let reset = self.head;

        if !self.can_recurse() {
            self.emit_text(markup);
            self.head -= 1;
            return Ok(());
        }

        let mut cell = self.parse(Some(
            contexts::TABLE_OPEN
                | contexts::TABLE_CELL_OPEN
                | line_context
                | contexts::TABLE_CELL_STYLE,
        ), None)?;

        let cell_context = self.context();
        *self.context_mut() = old_context;
        let reset_for_style = cell_context & contexts::TABLE_CELL_STYLE != 0;

        if reset_for_style {
            self.head = reset;
            self.push(Some(
                contexts::TABLE_OPEN | contexts::TABLE_CELL_OPEN | line_context,
            ))?;
            padding = self.handle_table_style("|")?;
            style = self.pop(None);
            self.head += 1; // Skip the style separator
            cell = self.parse(Some(
                contexts::TABLE_OPEN | contexts::TABLE_CELL_OPEN | line_context,
            ), None)?;
            *self.context_mut() = old_context;
        }

        let close_open_markup = if reset_for_style { Some("|".to_string()) } else { None };
        self.emit_table_tag(markup, tag, style, padding, close_open_markup, cell, "".to_string());

        *self.context_mut() |= cell_context & (contexts::TABLE_TH_LINE | contexts::TABLE_TD_LINE);
        self.head -= 1; // Offset displacement done by parse()
        Ok(())
    }

    fn handle_table_cell_end(&mut self, reset_for_style: Option<bool>) -> Vec<Token> {
        let reset_for_style = reset_for_style.unwrap_or(false);
        if reset_for_style {
            *self.context_mut() |= contexts::TABLE_CELL_STYLE;
        } else {
            *self.context_mut() &= !contexts::TABLE_CELL_STYLE;
        }
        self.pop(Some(true))
    }

    fn handle_table_row_end(&mut self) -> Vec<Token> {
        self.head += 2;
        self.pop(None)
    }

    fn handle_table_end(&mut self) -> Vec<Token> {
        self.head += 2;
        self.pop(None)
    }

    /// Handle the end of the stream of wikitext
    fn handle_end(&mut self) -> Result<Vec<Token>, TokenizerError> {
        if self.context() & contexts::FAIL != 0 {
            if self.context() & contexts::TAG_BODY != 0 {
                if definitions::is_single(
                    &*self.stack()[1]
                        .get("text")
                        .cloned()
                        .unwrap_or(Value::String("".to_string()))
                        .unwrap_string(),
                ) {
                    return self.handle_single_tag_end();
                }
            }
            if self.context() & contexts::TABLE_CELL_OPEN != 0 {
                self.pop(None);
            }
            if self.context() & contexts::DOUBLE != 0 {
                self.pop(None);
            }
            return Err(self.fail_route());
        }
        Ok(self.pop(None))
    }

    /// Make sure we are not trying to write an invalid character.
    fn verify_safe(&mut self, this: Either<String, Sentinel>) -> bool {
        let context = self.context();
        if context & contexts::FAIL_NEXT != 0 {
            return false;
        }
        if context & contexts::WIKILINK_TITLE != 0 {
            match this {
                Either::Left(ref s) if s == "]" || s == "{" => {
                    *self.context_mut() |= contexts::FAIL_NEXT;
                }
                Either::Left(ref s) if s == "\n" || s == "[" || s == "}" || s == ">" => {
                    return false;
                }
                Either::Left(ref s) if s == "<" => {
                    if self
                        .read(Some(1), None)
                        .unwrap_or(Either::Right(Sentinel::End))
                        == Either::Left("!".to_string())
                    {
                        *self.context_mut() |= contexts::FAIL_NEXT;
                    } else {
                        return false;
                    }
                }
                _ => {}
            }
            return true;
        }
        if context & contexts::EXT_LINK_TITLE != 0 {
            return this != Either::Left("\n".to_string());
        }
        if context & contexts::TEMPLATE_NAME != 0 {
            match this {
                Either::Left(ref s) if s == "{" => {
                    *self.context_mut() |= contexts::HAS_TEMPLATE | contexts::FAIL_NEXT;
                    return true;
                }
                Either::Left(ref s)
                    if s == "}"
                        || (s == "<"
                            && self
                                .read(Some(1), None)
                                .unwrap_or(Either::Right(Sentinel::End))
                                == Either::Left("!".to_string())) =>
                {
                    *self.context_mut() |= contexts::FAIL_NEXT;
                    return true;
                }
                Either::Left(ref s) if s == "[" || s == "]" || s == "<" || s == ">" => {
                    return false;
                }
                Either::Left(ref s) if s == "|" => {
                    return true;
                }
                Either::Left(ref s) if context & contexts::HAS_TEXT != 0 => {
                    if context & contexts::FAIL_ON_TEXT != 0 {
                        if s == "END" || !s.trim().is_empty() {
                            return false;
                        }
                    } else if s == "\n" {
                        *self.context_mut() |= contexts::FAIL_ON_TEXT;
                    }
                }
                Either::Left(ref s) if s == "END" || !s.trim().is_empty() => {
                    *self.context_mut() |= contexts::HAS_TEXT;
                }
                _ => {}
            }
            return true;
        }
        if context & contexts::TAG_CLOSE != 0 {
            return this != Either::Left("<".to_string());
        }
        if context & contexts::FAIL_ON_EQUALS != 0 {
            if this == Either::Left("=".to_string()) {
                return false;
            }
        } else if context & contexts::FAIL_ON_LBRACE != 0 {
            match this {
                Either::Left(ref s)
                    if s == "{"
                        || (self
                            .read(Some(-1), None)
                            .unwrap_or(Either::Right(Sentinel::End))
                            == self
                                .read(Some(-2), None)
                                .unwrap_or(Either::Right(Sentinel::End))
                            && s == "{") =>
                {
                    if context & contexts::TEMPLATE != 0 {
                        *self.context_mut() |= contexts::FAIL_ON_EQUALS;
                    } else {
                        *self.context_mut() |= contexts::FAIL_NEXT;
                    }
                    return true;
                }
                _ => {
                    *self.context_mut() ^= contexts::FAIL_ON_LBRACE;
                }
            }
        } else if context & contexts::FAIL_ON_RBRACE != 0 {
            if this == Either::Left("}".to_string()) {
                *self.context_mut() |= contexts::FAIL_NEXT;
                return true;
            }
            *self.context_mut() ^= contexts::FAIL_ON_RBRACE;
        } else if this == Either::Left("{".to_string()) {
            *self.context_mut() |= contexts::FAIL_ON_LBRACE;
        } else if this == Either::Left("}".to_string()) {
            *self.context_mut() |= contexts::FAIL_ON_RBRACE;
        }
        true
    }

    fn parse(
        &mut self,
        context: Option<u64>,
        push: Option<bool>,
    ) -> Result<Vec<Token>, TokenizerError> {
        let context = context.unwrap_or(0);
        let push = push.unwrap_or(true);
        if push {
            self.push(Some(context))?;
        }
        loop {
            let this = self.read(None, None)?;
            if self.context() & contexts::UNSAFE != 0 {
                if !self.verify_safe(this.clone()) {
                    if self.context() & contexts::DOUBLE != 0 {
                        self.pop(None);
                    }
                    return Err(self.fail_route());
                }
            }
            if !&this
                .clone()
                .left()
                .map_or(true, |s| MARKERS.contains(&Marker::Str(&s)))
            {
                self.emit_text(this.unwrap_left());
                self.head += 1;
                continue;
            }
            if this == Either::Right(Sentinel::End) {
                return self.handle_end();
            }
            let nxt = self.read(Some(1), None)?;
            match this.clone().unwrap_left().as_str() {
                "{" if this == nxt => {
                    if self.can_recurse() {
                        self.parse_template_or_argument()?;
                    } else {
                        self.emit_text("{".to_string());
                    }
                }
                "|" if self.context() & contexts::TEMPLATE != 0 => self.handle_template_param()?,
                "=" if self.context() & contexts::TEMPLATE_PARAM_KEY != 0 => {
                    if self.global & contexts::GL_HEADING == 0
                        && vec![
                            Either::Left("\n".to_string()),
                            Either::Right(Sentinel::Start),
                        ]
                        .contains(&self.read(Some(-1), None)?)
                        && nxt == Either::Left("=".to_string())
                    {
                        self.parse_heading()?;
                    } else {
                        self.handle_template_param_value()?;
                    }
                }
                "}" if this == nxt && self.context() & contexts::TEMPLATE != 0 => {
                    return self.handle_template_end();
                }
                "|" if self.context() & contexts::ARGUMENT_NAME != 0 => {
                    self.handle_argument_separator()?
                }
                "}" if this == nxt && self.context() & contexts::ARGUMENT != 0 => {
                    if self.read(Some(2), None)? == Either::Left("}".to_string()) {
                        return self.handle_argument_end();
                    }
                    self.emit_text("}".to_string());
                }
                "[" if this == nxt && self.can_recurse() => {
                    if self.context() & contexts::NO_WIKILINKS == 0 {
                        self.parse_wikilink()?;
                    } else {
                        self.emit_text("[".to_string());
                    }
                }
                "|" if self.context() & contexts::WIKILINK_TITLE != 0 => {
                    self.handle_wikilink_separator()
                }
                "]" if this == nxt && self.context() & contexts::WIKILINK != 0 => {
                    return Ok(self.handle_wikilink_end());
                }
                "[" => self.parse_external_link(true)?,
                ":" if self.read(Some(-1), None)?.unwrap_left() != MARKERS[0].unwrap_str() => {
                    self.parse_external_link(false)?;
                }
                "]" if self.context() & contexts::EXT_LINK_TITLE != 0 => return Ok(self.pop(None)),
                "=" if self.global & contexts::GL_HEADING == 0
                    && self.context() & contexts::TEMPLATE == 0 =>
                {
                    let prev = self.read(Some(-1), None)?;
                    if vec![
                        Either::Left("\n".to_string()),
                        Either::Right(Sentinel::Start),
                    ]
                    .contains(&prev)
                    {
                        self.parse_heading()?;
                    } else {
                        self.emit_text("=".to_string());
                    }
                }
                "=" if self.context() & contexts::HEADING != 0 => {
                    // IMPORTANT: THIS IS A HACK TO TRY TO KEEP THE ARCHITECTURE IN LINE WITH PYTHON
                    // Heading is over
                    let (mut out, l) = self.handle_heading_end()?;
                    let mut tmp = Token::heading_start();
                    tmp.insert("level".to_string(), Value::Integer(l as i64));
                    out.push(tmp);
                    return Ok(out);
                }
                "\n" if self.context() & contexts::HEADING != 0 => return Err(self.fail_route()),
                "&" => self.parse_entity()?,
                "<" if nxt.clone() == Either::Left("!".to_string()) => {
                    if self.read(Some(2), None)? == self.read(Some(3), None)?
                        && self.read(Some(2), None)? == Either::Left("-".to_string())
                    {
                        self.parse_comment()?;
                    } else {
                        self.emit_text("<".to_string());
                    }
                }
                "<" if nxt.clone().unwrap_left() == "/"
                    && self.read(Some(2), None)? != Either::Right(Sentinel::End) =>
                {
                    if self.context() & contexts::TAG_BODY != 0 {
                        self.handle_tag_open_close()?;
                    } else {
                        self.handle_invalid_tag_start()?;
                    }
                }
                "<" if self.context() & contexts::TAG_CLOSE == 0 => {
                    if self.can_recurse() {
                        self.parse_tag()?;
                    } else {
                        self.emit_text("<".to_string());
                    }
                }
                ">" if self.context() & contexts::TAG_CLOSE != 0 => {
                    return Ok(self.handle_tag_close_close()?);
                }
                "'" if this == nxt && !self.skip_style_tags => {
                    if let Some(result) = self.parse_style()? {
                        return Ok(result);
                    }
                }
                "#" | "*" | ";" | ":" if self.read(Some(-1), None)?.unwrap_left() == "\n" => {
                    self.handle_list()?;
                }
                "-" if this == nxt
                    && self.read(Some(2), None)? == this
                    && self.read(Some(3), None)? == this =>
                {
                    self.handle_hr()?;
                }
                "\n" | ":" if self.context() & contexts::DL_TERM != 0 => {
                    self.handle_dl_term()?;
                    if this == Either::Left("\n".to_string()) {
                        *self.context_mut() &= !contexts::TABLE_CELL_LINE_CONTEXTS;
                    }
                }
                "{" if nxt.clone().unwrap_left() == "|" => {
                    if self.can_recurse() {
                        self.parse_table()?;
                    } else {
                        self.emit_text("{".to_string());
                    }
                }
                "|" if self.context() & contexts::TABLE_OPEN != 0 => {
                    if nxt.clone().unwrap_left() == "|"
                        && self.context() & contexts::TABLE_TD_LINE != 0
                    {
                        if self.context() & contexts::TABLE_CELL_OPEN != 0 {
                            return Ok(self.handle_table_cell_end(None));
                        }
                        self.handle_table_cell(
                            "||".to_string(),
                            "td".to_string(),
                            contexts::TABLE_TD_LINE,
                        )?;
                    } else if nxt.clone().unwrap_left() == "|"
                        && self.context() & contexts::TABLE_TH_LINE != 0
                    {
                        if self.context() & contexts::TABLE_CELL_OPEN != 0 {
                            return Ok(self.handle_table_cell_end(None));
                        }
                        self.handle_table_cell(
                            "||".to_string(),
                            "th".to_string(),
                            contexts::TABLE_TH_LINE,
                        )?;
                    } else if nxt.clone().unwrap_left() == "!"
                        && self.context() & contexts::TABLE_TH_LINE != 0
                    {
                        if self.context() & contexts::TABLE_CELL_OPEN != 0 {
                            return Ok(self.handle_table_cell_end(None));
                        }
                        self.handle_table_cell(
                            "!!".to_string(),
                            "th".to_string(),
                            contexts::TABLE_TH_LINE,
                        )?;
                    } else if this.clone().unwrap_left() == "|"
                        && self.context() & contexts::TABLE_CELL_STYLE != 0
                    {
                        return Ok(self.handle_table_cell_end(Some(true)));
                    } else if this.clone().unwrap_left() == "\n"
                        && self.context() & contexts::TABLE_CELL_LINE_CONTEXTS != 0
                    {
                        *self.context_mut() &= !contexts::TABLE_CELL_LINE_CONTEXTS;
                        self.emit_text("\n".to_string());
                    } else if self.read(Some(-1), None)?.unwrap_left() == "\n" {
                        if this.clone().unwrap_left() == "|" && nxt.clone().unwrap_left() == "}" {
                            if self.context() & contexts::TABLE_CELL_OPEN != 0 {
                                return Ok(self.handle_table_cell_end(None));
                            }
                            if self.context() & contexts::TABLE_ROW_OPEN != 0 {
                                return Ok(self.handle_table_row_end());
                            }
                            return Ok(self.handle_table_end());
                        }
                        if this.clone().unwrap_left() == "|" && nxt.unwrap_left() == "-" {
                            if self.context() & contexts::TABLE_CELL_OPEN != 0 {
                                return Ok(self.handle_table_cell_end(None));
                            }
                            if self.context() & contexts::TABLE_ROW_OPEN != 0 {
                                return Ok(self.handle_table_row_end());
                            }
                            self.handle_table_row();
                        } else if this.clone().unwrap_left() == "|" {
                            if self.context() & contexts::TABLE_CELL_OPEN != 0 {
                                return Ok(self.handle_table_cell_end(None));
                            }
                            self.handle_table_cell(
                                "|".to_string(),
                                "td".to_string(),
                                contexts::TABLE_TD_LINE,
                            )?;
                        } else if this.unwrap_left() == "!" {
                            if self.context() & contexts::TABLE_CELL_OPEN != 0 {
                                return Ok(self.handle_table_cell_end(None));
                            }
                            self.handle_table_cell(
                                "!".to_string(),
                                "th".to_string(),
                                contexts::TABLE_TH_LINE,
                            )?;
                        }
                    }
                }
                _ => {}
            }
            self.head += 1;
        }
    }

    pub fn tokenize(
        &mut self,
        text: String,
        context: u64,
        skip_style_tags: bool,
    ) -> Result<Vec<Token>, TokenizerError> {
        // Build a list of tokens from a string of wikicode and return it.
        let split: Vec<_> = split_with_captures(&Self::regex(), &text)
            .into_iter()
            .filter(|segment| !segment.is_empty())
            .map(String::from)
            .collect();
        self.text = split;
        self.head = 0;
        self.global = 0;
        self.depth = 0;
        self.bad_routes.clear();
        self.skip_style_tags = skip_style_tags;

        self.parse(Some(context), None).map_err(|exc| {
            if !self.stacks.is_empty() {
                return TokenizerError::NonEmptyExitStack;
            }
            exc
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        tokenizer::Tokenizer,
        tokens::{Token, TokenType},
    };

    #[test]
    fn test_basic() {
        let mut tokenizer = Tokenizer::new();
        let out = tokenizer
            .tokenize("Hello, world!".to_string(), 0, false)
            .unwrap();
        dbg!(&out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].token_type, TokenType::Text);
    }
}
