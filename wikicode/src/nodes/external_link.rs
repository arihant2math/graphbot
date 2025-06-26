use std::any::Any;

use crate::{nodes::_base::Node, utils::parse_anything, wikicode::Wikicode};

pub struct ExternalLink {
    url: Wikicode,
    title: Option<Wikicode>,
    brackets: bool,
    suppress_space: bool,
}

impl ExternalLink {
    pub fn new<U, T>(url: U, title: Option<T>, brackets: bool, suppress_space: bool) -> Self
    where
        U: Into<Wikicode>,
        T: Into<Wikicode>,
    {
        ExternalLink {
            url: url.into(),
            title: title.map(|t| t.into()),
            brackets,
            suppress_space,
        }
    }

    pub fn url(&self) -> &Wikicode {
        &self.url
    }

    pub fn set_url<V: Into<Wikicode>>(&mut self, value: V) {
        // If you have a context for EXT_LINK_URI, pass it here.
        self.url = parse_anything(value, Some(crate::parser::contexts::EXT_LINK_URI));
    }

    pub fn title(&self) -> Option<&Wikicode> {
        self.title.as_ref()
    }

    pub fn set_title<V: Into<Option<Wikicode>>>(&mut self, value: V) {
        self.title = value.into();
    }

    pub fn brackets(&self) -> bool {
        self.brackets
    }

    pub fn set_brackets(&mut self, value: bool) {
        self.brackets = value;
    }

    pub fn suppress_space(&self) -> bool {
        self.suppress_space
    }

    pub fn set_suppress_space(&mut self, value: bool) {
        self.suppress_space = value;
    }
}

impl Node for ExternalLink {
    fn as_str(&self) -> String {
        if self.brackets {
            match &self.title {
                Some(title) => {
                    if self.suppress_space {
                        format!("[{}{}]", self.url.as_str(), title.as_str())
                    } else {
                        format!("[{} {}]", self.url.as_str(), title.as_str())
                    }
                }
                None => format!("[{}]", self.url.as_str()),
            }
        } else {
            self.url.as_str()
        }
    }

    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Wikicode> + 'a> {
        match &self.title {
            Some(title) => Box::new(vec![&self.url, title].into_iter()),
            None => Box::new(std::iter::once(&self.url)),
        }
    }

    fn strip(&self, normalize: bool, collapse: bool, keep_template_params: bool) -> Option<String> {
        if self.brackets {
            if let Some(title) = &self.title {
                return Some(title.strip_code(normalize, collapse, keep_template_params));
            }
            None
        } else {
            Some(
                self.url
                    .strip_code(normalize, collapse, keep_template_params),
            )
        }
    }

    fn showtree(&self, write: &dyn Fn(&[&str]), get: &dyn Fn(&Wikicode), _mark: &dyn Fn()) {
        if self.brackets {
            write(&["["]);
        }
        get(&self.url);
        if let Some(title) = &self.title {
            get(title);
        }
        if self.brackets {
            write(&["]"]);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
