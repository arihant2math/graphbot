use std::any::Any;

use crate::nodes::_base::Node;

/// Represents a hidden HTML comment, like `<!-- foobar -->`.
pub struct Comment {
    contents: String,
}

impl Comment {
    pub fn new<S: ToString>(contents: S) -> Self {
        Comment {
            contents: contents.to_string(),
        }
    }

    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn set_contents<S: ToString>(&mut self, value: S) {
        self.contents = value.to_string();
    }
}

impl Node for Comment {
    fn as_str(&self) -> String {
        format!("<!--{}-->", self.contents)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
