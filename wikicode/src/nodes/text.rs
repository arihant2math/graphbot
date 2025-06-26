use std::any::Any;

use crate::nodes::_base::Node;

/// Represents ordinary, unformatted text with no special properties.
pub struct Text {
    value: String,
}

impl Text {
    pub fn new<V: ToString>(value: V) -> Self {
        Text {
            value: value.to_string(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value<V: ToString>(&mut self, newval: V) {
        self.value = newval.to_string();
    }
}

impl Node for Text {
    fn as_str(&self) -> String {
        self.value.clone()
    }

    fn strip(
        &self,
        _normalize: bool,
        _collapse: bool,
        _keep_template_params: bool,
    ) -> Option<String> {
        Some(self.value.clone())
    }

    fn showtree(
        &self,
        write: &dyn Fn(&[&str]),
        _get: &dyn Fn(&crate::wikicode::Wikicode),
        _mark: &dyn Fn(),
    ) {
        // Write the value as a unicode-escaped string, similar to Python's
        // encode("unicode_escape")
        let escaped = self.value.escape_default().to_string();
        write(&[&escaped]);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
