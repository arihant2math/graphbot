use std::any::Any;

use crate::{nodes::_base::Node, utils::parse_anything, wikicode::Wikicode};

/// Represents a section heading in wikicode, like `== Foo ==`.
pub struct Heading {
    title: Wikicode,
    level: u8,
}

impl Heading {
    pub fn new<T: Into<Wikicode>>(title: T, level: u8) -> Self {
        if level < 1 || level > 6 {
            panic!("Heading level must be between 1 and 6, got {}", level);
        }
        Heading {
            title: title.into(),
            level,
        }
    }

    pub fn title(&self) -> &Wikicode {
        &self.title
    }

    pub fn set_title<T: Into<Wikicode>>(&mut self, value: T) {
        self.title = value.into();
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn set_level(&mut self, value: u8) {
        if value < 1 || value > 6 {
            panic!("Heading level must be between 1 and 6, got {}", value);
        }
        self.level = value;
    }
}

impl Node for Heading {
    fn as_str(&self) -> String {
        let eqs = "=".repeat(self.level as usize);
        format!("{}{}{}", eqs, self.title.as_str(), eqs)
    }

    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Wikicode> + 'a> {
        Box::new(std::iter::once(&self.title))
    }

    fn strip(&self, normalize: bool, collapse: bool, keep_template_params: bool) -> Option<String> {
        Some(
            self.title
                .strip_code(normalize, collapse, keep_template_params),
        )
    }

    fn showtree(&self, write: &dyn Fn(&[&str]), get: &dyn Fn(&Wikicode), _mark: &dyn Fn()) {
        let eqs = "=".repeat(self.level as usize);
        write(&[&eqs]);
        get(&self.title);
        write(&[&eqs]);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
