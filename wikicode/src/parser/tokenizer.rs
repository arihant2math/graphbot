use std::collections::HashMap;

use crate::parser::tokens::Token;

struct BadRoute;

struct _TagOpenData {
    context: u64,
    padding_buffer: HashMap<String, String>,
    quoter: Option<char>,
    reset: u64,
}

const CX_NAME: u64 = 1 << 0;
const CX_ATTR_READY: u64 = 1 << 1;
const CX_ATTR_NAME: u64 = 1 << 2;
const CX_ATTR_VALUE: u64 = 1 << 3;
const CX_QUOTED: u64 = 1 << 4;
const CX_NOTE_SPACE: u64 = 1 << 5;
const CX_NOTE_EQUALS: u64 = 1 << 6;
const CX_NOTE_QUOTE: u64 = 1 << 7;

const MARKERS: [char; 19] = [
    '{', '}', '[', ']', '<', '>', '|', '=', '&', '\'', '"', '#', '*', ';', ':', '/', '-', '!', '\n',
];
const URISCHEME: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+.-";
const MAX_DEPTH: usize = 100;
// const MARKER_REGEX: &str = r"([{}\[\]<>|=&'#*;:/\\\"\-!\n])";
// const TAG_SPLITTER: &str = r"([\s\"\'\\]+)";

pub struct Tokenizer {
    text: String,
    head: usize,
    stacks: Vec<Vec<Token>>,
    global: u64,
    depth: usize,
    bad_routes: HashMap<String, BadRoute>,
    skip_style_tags: bool,
}

impl Tokenizer {
    pub fn new(text: String) -> Self {
        Tokenizer {
            text,
            head: 0,
            stacks: Vec::new(),
            global: 0,
            depth: 0,
            bad_routes: HashMap::new(),
            skip_style_tags: false,
        }
    }

    fn stack(&self) -> &Token {
        &self.stacks.last().unwrap()[0]
    }

    fn context(&self) -> &Token {
        &self.stacks.last().unwrap()[1]
    }

    fn set_context(&mut self, context: u64) {
        if let Some(stack) = self.stacks.last_mut() {
            if stack.len() > 1 {
                stack[1].set("context", context.to_string());
            } else {
                stack.push(Token::new());
                stack[1].set("context", context.to_string());
            }
        }
    }
}
