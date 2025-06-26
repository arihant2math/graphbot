use std::{any::Any, fmt};

use crate::{nodes::_base::Node, utils::parse_anything, wikicode::Wikicode};

/// Represents a template argument substitution, like `{{{foo}}}`.
pub struct Argument {
    name: Wikicode,
    default: Option<Wikicode>,
}

impl Argument {
    pub fn new<N: Into<Wikicode>, D: Into<Option<Wikicode>>>(name: N, default: D) -> Self {
        Argument {
            name: name.into(),
            default: default.into(),
        }
    }

    pub fn name(&self) -> &Wikicode {
        &self.name
    }

    pub fn set_name<V: Into<Wikicode>>(&mut self, value: V) {
        self.name = value.into();
    }

    pub fn default(&self) -> Option<&Wikicode> {
        self.default.as_ref()
    }

    pub fn set_default<V: Into<Option<Wikicode>>>(&mut self, value: V) {
        self.default = value.into();
    }
}

impl Node for Argument {
    fn as_str(&self) -> String {
        let start = format!("{{{{{{{}}}", self.name.as_str());
        match &self.default {
            Some(def) => format!("{}|{}}}}}", start, def.as_str()),
            None => format!("{}}}}}", start),
        }
    }

    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Wikicode> + 'a> {
        match &self.default {
            Some(def) => Box::new(vec![&self.name, def].into_iter()),
            None => Box::new(std::iter::once(&self.name)),
        }
    }

    fn strip(&self, normalize: bool, collapse: bool, keep_template_params: bool) -> Option<String> {
        self.default
            .as_ref()
            .map(|d| d.strip_code(normalize, collapse, keep_template_params))
    }

    fn showtree(&self, write: &dyn Fn(&[&str]), get: &dyn Fn(&Wikicode), mark: &dyn Fn()) {
        write(&["{{{"]);
        get(&self.name);
        if let Some(def) = &self.default {
            write(&["    | "]);
            mark();
            get(def);
        }
        write(&["}}}"]);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
