use crate::{
    nodes::{
        Argument, Comment, ExternalLink, HTMLEntity, Heading, Node, Tag, Template, Text, Wikilink,
        extras::{Attribute, Parameter},
    },
    parser::{
        errors::ParserError,
        tokens::{self, Token},
    },
    wikicode::Wikicode,
};

pub struct Builder {
    tokens: Vec<Token>,
    stacks: Vec<Vec<Box<dyn Node>>>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            tokens: Vec::new(),
            stacks: Vec::new(),
        }
    }

    fn push(&mut self) {
        self.stacks.push(Vec::new());
    }

    fn pop(&mut self) -> Wikicode {
        Wikicode::new(self.stacks.pop().unwrap())
    }

    fn write(&mut self, item: Box<dyn Node>) {
        if let Some(stack) = self.stacks.last_mut() {
            stack.push(item);
        }
    }

    fn handle_parameter(&mut self, default: i32) -> Result<Parameter, ParserError> {
        let mut key = None;
        let mut showkey = false;
        self.push();
        while let Some(token) = self.tokens.pop() {
            match token {
                Token::TemplateParamEquals => {
                    key = Some(self.pop());
                    showkey = true;
                    self.push();
                }
                Token::TemplateParamSeparator | Token::TemplateClose => {
                    self.tokens.push(token);
                    let value = self.pop();
                    if key.is_none() {
                        key = Some(Wikicode::new(vec![Box::new(Text::new(
                            default.to_string(),
                        ))]));
                    }
                    return Ok(Parameter::new(key.unwrap(), value, showkey));
                }
                _ => self.write(self.handle_token(token)?),
            }
        }
        Err(ParserError::new("_handle_parameter() missed a close token"))
    }

    fn handle_template(&mut self, _token: Token) -> Result<Template, ParserError> {
        let mut params = Vec::new();
        let mut default = 1;
        self.push();
        while let Some(token) = self.tokens.pop() {
            match token {
                Token::TemplateParamSeparator => {
                    if params.is_empty() {
                        let name = self.pop();
                        params.push(self.handle_parameter(default)?);
                        if !params.last().unwrap().showkey {
                            default += 1;
                        }
                    }
                }
                Token::TemplateClose => {
                    if params.is_empty() {
                        let name = self.pop();
                        return Ok(Template::new(name, params));
                    }
                }
                _ => self.write(self.handle_token(token)?),
            }
        }
        Err(ParserError::new("_handle_template() missed a close token"))
    }

    fn handle_token(&mut self, token: Token) -> Result<Box<dyn Node>, ParserError> {
        match token {
            Token::Text(text) => Ok(Box::new(Text::new(text))),
            // Add other token handlers here
            _ => Err(ParserError::new("_handle_token() got unexpected token")),
        }
    }

    pub fn build(&mut self, tokenlist: Vec<Token>) -> Result<Wikicode, ParserError> {
        self.tokens = tokenlist;
        self.tokens.reverse();
        self.push();
        while let Some(token) = self.tokens.pop() {
            let node = self.handle_token(token)?;
            self.write(node);
        }
        Ok(self.pop())
    }
}
