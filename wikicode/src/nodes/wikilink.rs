use std::any::Any;

use crate::{nodes::_base::Node, utils::parse_anything, wikicode::Wikicode};

/// Represents an internal wikilink, like `[[Foo|Bar]]`.
pub struct Wikilink {
    title: Wikicode,
    text: Option<Wikicode>,
}

impl Wikilink {
    pub fn new<T: Into<Wikicode>, X: Into<Option<Wikicode>>>(title: T, text: X) -> Self {
        Wikilink {
            title: title.into(),
            text: text.into(),
        }
    }

    pub fn title(&self) -> &Wikicode {
        &self.title
    }

    pub fn set_title<V: Into<Wikicode>>(&mut self, value: V) {
        self.title = value.into();
    }

    pub fn text(&self) -> Option<&Wikicode> {
        self.text.as_ref()
    }

    pub fn set_text<V: Into<Option<Wikicode>>>(&mut self, value: V) {
        self.text = value.into();
    }
}

impl Node for Wikilink {
    fn as_str(&self) -> String {
        match &self.text {
            Some(text) => format!("[[{}|{}]]", self.title.as_str(), text.as_str()),
            None => format!("[[{}]]", self.title.as_str()),
        }
    }

    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Wikicode> + 'a> {
        match &self.text {
            Some(text) => Box::new(vec![&self.title, text].into_iter()),
            None => Box::new(std::iter::once(&self.title)),
        }
    }

    fn strip(&self, normalize: bool, collapse: bool, keep_template_params: bool) -> Option<String> {
        match &self.text {
            Some(text) => Some(text.strip_code(normalize, collapse, keep_template_params)),
            None => Some(
                self.title
                    .strip_code(normalize, collapse, keep_template_params),
            ),
        }
    }

    fn showtree(&self, write: &dyn Fn(&[&str]), get: &dyn Fn(&Wikicode), mark: &dyn Fn()) {
        write(&["[["]);
        get(&self.title);
        if let Some(text) = &self.text {
            write(&["    | "]);
            mark();
            get(text);
        }
        write(&["]]"]);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
