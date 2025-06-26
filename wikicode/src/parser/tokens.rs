use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    attributes: HashMap<String, String>,
}

impl Token {
    pub fn new() -> Self {
        Token {
            attributes: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    pub fn set(&mut self, key: &str, value: String) {
        self.attributes.insert(key.to_string(), value);
    }

    pub fn delete(&mut self, key: &str) {
        self.attributes.remove(key);
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = Vec::new();
        for (key, value) in &self.attributes {
            if value.len() > 100 {
                args.push(format!("{}={:?}...", key, &value[..97]));
            } else {
                args.push(format!("{}={:?}", key, value));
            }
        }
        write!(f, "{}({})", "Token", args.join(", "))
    }
}

macro_rules! define_token {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name(Token);

        impl $name {
            pub fn new() -> Self {
                $name(Token::new())
            }
        }
    };
}

// Define token types
define_token!(Text);

define_token!(TemplateOpen);
define_token!(TemplateParamSeparator);
define_token!(TemplateParamEquals);
define_token!(TemplateClose);

define_token!(ArgumentOpen);
define_token!(ArgumentSeparator);
define_token!(ArgumentClose);

define_token!(WikilinkOpen);
define_token!(WikilinkSeparator);
define_token!(WikilinkClose);

define_token!(ExternalLinkOpen);
define_token!(ExternalLinkSeparator);
define_token!(ExternalLinkClose);

define_token!(HTMLEntityStart);
define_token!(HTMLEntityNumeric);
define_token!(HTMLEntityHex);
define_token!(HTMLEntityEnd);

define_token!(HeadingStart);
define_token!(HeadingEnd);

define_token!(CommentStart);
define_token!(CommentEnd);

define_token!(TagOpenOpen);
define_token!(TagAttrStart);
define_token!(TagAttrEquals);
define_token!(TagAttrQuote);
define_token!(TagCloseOpen);
define_token!(TagCloseSelfclose);
define_token!(TagOpenClose);
define_token!(TagCloseClose);
