use regex::Regex;

use crate::{utils::parse_anything, wikicode::Wikicode};

/// Represents a parameter of a template.
/// For example, the template `{{foo|bar|spam=eggs}}` contains two
/// Parameters: one whose name is `"1"`, value is `"bar"`, and `showkey`
/// is `false`, and one whose name is `"spam"`, value is `"eggs"`, and
/// `showkey` is `true`.
pub struct Parameter {
    name: Wikicode,
    value: Wikicode,
    showkey: bool,
}

impl Parameter {
    pub fn new<N: Into<Wikicode>, V: Into<Wikicode>>(name: N, value: V, showkey: bool) -> Self {
        let name_wc = name.into();
        if !showkey && !Self::can_hide_key(&name_wc) {
            panic!("parameter key {:?} cannot be hidden", name_wc.as_str());
        }
        Parameter {
            name: name_wc,
            value: value.into(),
            showkey,
        }
    }

    pub fn name(&self) -> &Wikicode {
        &self.name
    }

    pub fn set_name<N: Into<Wikicode>>(&mut self, newval: N) {
        self.name = newval.into();
    }

    pub fn value(&self) -> &Wikicode {
        &self.value
    }

    pub fn set_value<V: Into<Wikicode>>(&mut self, newval: V) {
        self.value = newval.into();
    }

    pub fn showkey(&self) -> bool {
        self.showkey
    }

    pub fn set_showkey(&mut self, newval: bool) {
        if !newval && !Self::can_hide_key(&self.name) {
            panic!("parameter key {:?} cannot be hidden", self.name.as_str());
        }
        self.showkey = newval;
    }

    /// Return whether or not the given key can be hidden.
    pub fn can_hide_key(key: &Wikicode) -> bool {
        // Matches r"[1-9][0-9]*$"
        static RE: once_cell::sync::Lazy<Regex> =
            once_cell::sync::Lazy::new(|| Regex::new(r"^[1-9][0-9]*$").unwrap());
        RE.is_match(key.as_str().trim())
    }
}

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.showkey {
            write!(f, "{}={}", self.name.as_str(), self.value.as_str())
        } else {
            write!(f, "{}", self.value.as_str())
        }
    }
}
