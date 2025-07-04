use std::fmt;

use crate::{utils::parse_anything, wikicode::Wikicode};

#[derive(Clone, Debug)]
pub struct Attribute {
    name: Wikicode,
    value: Option<Wikicode>,
    quotes: Option<String>,
    pad_first: String,
    pad_before_eq: String,
    pad_after_eq: String,
}

impl Attribute {
    pub fn new(
        name: impl Into<Wikicode>,
        value: Option<impl Into<Wikicode>>,
        quotes: Option<&str>,
        pad_first: &str,
        pad_before_eq: &str,
        pad_after_eq: &str,
    ) -> Self {
        let mut attr = Attribute {
            name: parse_anything(name, 0, false),
            value: None,
            quotes: None,
            pad_first: pad_first.to_string(),
            pad_before_eq: pad_before_eq.to_string(),
            pad_after_eq: pad_after_eq.to_string(),
        };
        attr.set_value_opt(value.map(|v| v.into()));
        attr.set_quotes_opt(quotes);
        attr
    }

    pub fn name(&self) -> &Wikicode {
        &self.name
    }

    pub fn set_name(&mut self, value: impl Into<Wikicode>) {
        self.name = parse_anything(value, 0, false);
    }

    pub fn value(&self) -> Option<&Wikicode> {
        self.value.as_ref()
    }

    pub fn set_value(&mut self, value: impl Into<Wikicode>) {
        let code = parse_anything(value, 0, false);
        let quotes = Self::value_needs_quotes(&code);
        if let Some(qs) = &quotes {
            if self.quotes.is_none() || !qs.contains(self.quotes.as_ref().unwrap()) {
                self.quotes = Some(qs.chars().next().unwrap().to_string());
            }
        }
        self.value = Some(code);
    }

    pub fn set_value_opt(&mut self, value: Option<impl Into<Wikicode>>) {
        if let Some(v) = value {
            self.set_value(v);
        } else {
            self.value = None;
        }
    }

    pub fn quotes(&self) -> Option<&str> {
        self.quotes.as_deref()
    }

    pub fn set_quotes(&mut self, value: &str) {
        let coerced = Self::coerce_quotes(Some(value)).unwrap();
        if coerced.is_empty()
            && self
                .value
                .as_ref()
                .map_or(false, |v| Self::value_needs_quotes(v).is_some())
        {
            panic!("attribute value requires quotes");
        }
        self.quotes = if coerced.is_empty() {
            None
        } else {
            Some(coerced)
        };
    }

    pub fn set_quotes_opt(&mut self, value: Option<&str>) {
        if let Some(v) = value {
            self.set_quotes(v);
        }
    }

    pub fn pad_first(&self) -> &str {
        &self.pad_first
    }

    pub fn set_pad_first(&mut self, value: &str) {
        self.set_padding(&mut self.pad_first, value);
    }

    pub fn pad_before_eq(&self) -> &str {
        &self.pad_before_eq
    }

    pub fn set_pad_before_eq(&mut self, value: &str) {
        self.set_padding(&mut self.pad_before_eq, value);
    }

    pub fn pad_after_eq(&self) -> &str {
        &self.pad_after_eq
    }

    pub fn set_pad_after_eq(&mut self, value: &str) {
        self.set_padding(&mut self.pad_after_eq, value);
    }

    fn set_padding(&mut self, field: &mut String, value: &str) {
        if value.is_empty() {
            *field = "".to_string();
        } else if !value.chars().all(|c| c.is_whitespace()) {
            panic!("padding must be entirely whitespace");
        } else {
            *field = value.to_string();
        }
    }

    pub fn coerce_quotes(quotes: Option<&str>) -> Option<String> {
        match quotes {
            None | Some("") => None,
            Some("\"") | Some("'") => Some(quotes.unwrap().to_string()),
            Some(q) => panic!("{:?} is not a valid quote type", q),
        }
    }

    pub fn value_needs_quotes(value: &Wikicode) -> Option<&'static str> {
        if value.nodes().is_empty() {
            return None;
        }
        let val = value
            // TODO: investigate why false is needed here
            // .filter_text(false)
            .filter_text()
            .iter()
            .map(|n| n.as_str())
            .collect::<String>();
        if !val.chars().any(|c| c.is_whitespace()) {
            return None;
        }
        let has_single = val.contains('\'');
        let has_double = val.contains('"');
        match (has_single, has_double) {
            (true, false) => Some("\""),
            (false, true) => Some("'"),
            (true, true) => Some("\"'"),
            (false, false) => Some("\"'"),
        }
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        result.push_str(&self.pad_first);
        result.push_str(&self.name.as_str());
        result.push_str(&self.pad_before_eq);
        if let Some(val) = &self.value {
            result.push('=');
            result.push_str(&self.pad_after_eq);
            if let Some(q) = &self.quotes {
                result.push_str(q);
                result.push_str(&val.as_str());
                result.push_str(q);
            } else {
                result.push_str(&val.as_str());
            }
        }
        write!(f, "{}", result)
    }
}
