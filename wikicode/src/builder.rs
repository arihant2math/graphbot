use crate::{
    nodes,
    nodes::{
        CommentNode, ExternalLinkNode, HeadingNode, Node, ParameterNode, TemplateNode, TextNode,
        Wikicode, WikilinkNode,
    },
    tokens::{Token, TokenType},
};

#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    MissedCloseToken(TokenType),
    UnsupportedEntry(TokenType),
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::MissedCloseToken(token_type) => {
                write!(f, "Missed close token: {:?}", token_type)
            }
            BuilderError::UnsupportedEntry(token_type) => {
                write!(f, "Unsupported entry: {:?}", token_type)
            }
        }
    }
}

pub struct Builder {
    tokens: Vec<Token>,
    stacks: Vec<Vec<Box<dyn Node>>>,
}

macro_rules! handle_and_write {
    ($s:ident, $token:ident) => {{
        let t = $s.handle_token($token);
        match t {
            Ok(t) => $s.write(t),
            Err(e) => return Err(e),
        }
    }};
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

    fn pop(&mut self) -> nodes::Wikicode {
        nodes::Wikicode {
            nodes: self.stacks.pop().unwrap(),
        }
    }

    fn write(&mut self, item: Box<dyn Node>) {
        if let Some(stack) = self.stacks.last_mut() {
            stack.push(item);
        } else {
            panic!("No stack to write to");
        }
    }

    fn handle_parameter(&mut self, default: String) -> Result<ParameterNode, BuilderError> {
        let mut key = None;
        let mut showkey = false;
        self.push();
        while let Some(token) = self.tokens.pop() {
            if matches!(token.token_type, TokenType::TemplateParamEquals) {
                key = Some(self.pop());
                showkey = true;
                self.push();
            } else if matches!(token.token_type, TokenType::TemplateParamSeparator) {
                self.tokens.push(token);
                let value = Some(self.pop());
                if key.is_none() {
                    key = Some(Wikicode::from_nodes(vec![Box::new(TextNode {
                        value: default,
                    })]));
                }
                return Ok(ParameterNode {
                    key: key.unwrap(),
                    showkey,
                    value,
                });
            }
        }
        Err(BuilderError::MissedCloseToken(
            TokenType::TemplateParamSeparator,
        ))
    }

    fn handle_token(&mut self, token: Token) -> Result<Box<dyn Node>, BuilderError> {
        match token.token_type {
            TokenType::Text => Ok(Box::new(TextNode {
                value: token.get("text").unwrap().clone().unwrap_string(),
            })),
            TokenType::TemplateOpen => {
                let mut name = None;
                let mut params = Vec::new();
                let mut default = 1;
                self.push();
                while let Some(token) = self.tokens.pop() {
                    if matches!(token.token_type, TokenType::TemplateParamSeparator) {
                        if params.is_empty() {
                            name = Some(self.pop());
                        }
                        let param = self.handle_parameter(default.to_string())?;
                        if !param.showkey {
                            default += 1;
                        }
                        params.push(param);
                    } else if matches!(token.token_type, TokenType::TemplateClose) {
                        if params.is_empty() {
                            name = Some(self.pop());
                        }
                        let name = name.unwrap();
                        return Ok(Box::new(TemplateNode { name, params }));
                    } else {
                        handle_and_write!(self, token);
                    }
                }
                Err(BuilderError::MissedCloseToken(TokenType::TemplateClose))
            }
            TokenType::ArgumentOpen => {
                unimplemented!()
            }
            TokenType::ArgumentSeparator => {
                unimplemented!()
            }
            TokenType::ArgumentClose => {
                unimplemented!()
            }
            TokenType::WikilinkOpen => {
                let mut title = None;
                self.push();
                while let Some(token) = self.tokens.pop() {
                    if matches!(token.token_type, TokenType::WikilinkSeparator) {
                        title = Some(self.pop());
                        self.push()
                    } else if matches!(token.token_type, TokenType::WikilinkClose) {
                        if !title.is_none() {
                            return Ok(Box::new(WikilinkNode {
                                title: title.unwrap(),
                                text: Some(self.pop()),
                            }));
                        }
                        return Ok(Box::new(WikilinkNode {
                            title: nodes::Wikicode { nodes: Vec::new() },
                            text: None,
                        }));
                    } else {
                        handle_and_write!(self, token);
                    }
                }
                Err(BuilderError::MissedCloseToken(TokenType::WikilinkClose))
            }
            TokenType::ExternalLinkOpen => {
                self.push();
                while let Some(token) = self.tokens.pop() {
                    let brackets = token.get("brackets").cloned().unwrap().unwrap_string();
                    let mut url = None;
                    let mut suppress_space = None;
                    if matches!(token.token_type, TokenType::ExternalLinkSeparator) {
                        url = Some(self.pop());
                        suppress_space = token
                            .get("suppress_space")
                            .cloned()
                            .map(|v| v.unwrap_string());
                        self.push();
                    } else if matches!(token.token_type, TokenType::ExternalLinkClose) {
                        if let Some(url) = url {
                            return Ok(Box::new(ExternalLinkNode {
                                url: url,
                                title: Some(self.pop()),
                                brackets,
                                suppress_space: suppress_space.is_some(),
                            }));
                        }
                        return Ok(Box::new(ExternalLinkNode {
                            url: self.pop(),
                            title: None,
                            brackets,
                            suppress_space: suppress_space.is_some(),
                        }));
                    } else {
                        handle_and_write!(self, token);
                    }
                }
                Err(BuilderError::MissedCloseToken(TokenType::ExternalLinkClose))
            }
            TokenType::HTMLEntityStart => {
                unimplemented!()
            }
            TokenType::HeadingStart => {
                let level = token.get("level").cloned().unwrap().unwrap_integer() as u8;
                self.push();
                while let Some(token) = self.tokens.pop() {
                    if matches!(token.token_type, TokenType::HeadingEnd) {
                        let title = self.pop();
                        return Ok(Box::new(HeadingNode { level, title }));
                    }
                    handle_and_write!(self, token);
                }
                Err(BuilderError::MissedCloseToken(TokenType::HeadingEnd))
            }
            TokenType::CommentStart => {
                self.push();
                while let Some(token) = self.tokens.pop() {
                    if matches!(token.token_type, TokenType::CommentEnd) {
                        return Ok(Box::new(CommentNode {
                            contents: self.pop(),
                        }));
                    }
                    handle_and_write!(self, token);
                }
                Err(BuilderError::MissedCloseToken(TokenType::CommentEnd))
            }
            TokenType::TagOpenOpen => {
                unimplemented!()
            }
            TokenType::TagAttrStart => {
                unimplemented!()
            }
            TokenType::TagAttrEquals => {
                unimplemented!()
            }
            TokenType::TagAttrQuote => {
                unimplemented!()
            }
            TokenType::TagCloseOpen => {
                unimplemented!()
            }
            TokenType::TagCloseSelfclose => {
                unimplemented!()
            }
            TokenType::TagOpenClose => {
                unimplemented!()
            }
            TokenType::TagCloseClose => {
                unimplemented!()
            }
            _ => Err(BuilderError::UnsupportedEntry(token.token_type)),
        }
    }

    pub fn build(&mut self, tokenlist: Vec<Token>) -> Result<nodes::Wikicode, BuilderError> {
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
