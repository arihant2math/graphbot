use std::any::Any;

use html_escape::decode_html_entities;

use crate::nodes::_base::Node;

/// Represents an HTML entity, like `&nbsp;`, either named or unnamed.
pub struct HTMLEntity {
    value: String,
    named: bool,
    hexadecimal: bool,
    hex_char: char,
}

impl HTMLEntity {
    /// Create a new HTML entity.
    /// * `value` – either a name or numeric string
    /// * `named` – if `Some`, forces named vs. numeric; if `None`, guesses
    /// * `hexadecimal` – if numeric, whether to treat as hex
    /// * `hex_char` – 'x' or 'X' for hex entities
    pub fn new<V: ToString>(
        value: V,
        named: Option<bool>,
        hexadecimal: bool,
        hex_char: char,
    ) -> Self {
        let s = value.to_string();
        let (named, hexadecimal) = if let Some(n) = named {
            (n, hexadecimal)
        } else if s.parse::<u32>().is_ok() {
            (false, false)
        } else if u32::from_str_radix(&s, 16).is_ok() {
            (false, true)
        } else {
            (true, false)
        };
        HTMLEntity {
            value: s,
            named,
            hexadecimal,
            hex_char,
        }
    }

    /// The raw entity value (name or number).
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set a new value and adjust `named`/`hexadecimal` flags by guessing.
    pub fn set_value<V: ToString>(&mut self, newval: V) {
        let s = newval.to_string();
        if s.parse::<u32>().is_ok() {
            self.named = false;
            self.hexadecimal = false;
        } else if u32::from_str_radix(&s, 16).is_ok() {
            self.named = false;
            self.hexadecimal = true;
        } else {
            self.named = true;
            self.hexadecimal = false;
        }
        self.value = s;
    }

    /// Whether this is a named entity (`&nbsp;`).
    pub fn named(&self) -> bool {
        self.named
    }

    /// Force named vs. numeric.
    pub fn set_named(&mut self, named: bool) {
        // (validation against known names could be added here)
        self.named = named;
    }

    /// If numeric, whether this is hex (`&#xFF;`) vs. decimal (`&#255;`).
    pub fn hexadecimal(&self) -> bool {
        self.hexadecimal
    }

    pub fn set_hexadecimal(&mut self, hex: bool) {
        self.hexadecimal = hex;
    }

    /// The letter used for hex notation ('x' or 'X').
    pub fn hex_char(&self) -> char {
        self.hex_char
    }

    pub fn set_hex_char(&mut self, ch: char) {
        if ch != 'x' && ch != 'X' {
            panic!("hex_char must be 'x' or 'X'");
        }
        self.hex_char = ch;
    }

    /// Decode the entity to its Unicode character(s).
    pub fn normalize(&self) -> String {
        let s = self.to_string();
        decode_html_entities(&s).to_string()
    }
}

impl std::fmt::Display for HTMLEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.named {
            write!(f, "&{};", self.value)
        } else if self.hexadecimal {
            write!(f, "&#{}{};", self.hex_char, self.value)
        } else {
            write!(f, "&#{};", self.value)
        }
    }
}

impl Node for HTMLEntity {
    fn as_str(&self) -> String {
        self.to_string()
    }

    fn strip(
        &self,
        normalize: bool,
        _collapse: bool,
        _keep_template_params: bool,
    ) -> Option<String> {
        if normalize {
            Some(self.normalize())
        } else {
            Some(self.to_string())
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
