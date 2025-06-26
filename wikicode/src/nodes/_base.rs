use std::{any::Any, fmt::Display};

pub trait Node: Any {
    /// Return a string representation of the node.
    fn as_str(&self) -> String {
        // By default, not implemented.
        unimplemented!("as_str() not implemented for this node type");
    }

    /// Return an iterator over child Wikicode objects, if any.
    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a crate::wikicode::Wikicode> + 'a> {
        // Default: yields nothing.
        Box::new(std::iter::empty())
    }

    /// Return the printable version of the node, or None if not printable.
    fn strip(&self, normalize: bool, collapse: bool, keep_template_params: bool) -> Option<String> {
        None
    }

    /// Build a tree representation of the node.
    fn showtree(
        &self,
        write: &dyn Fn(&[&str]),
        get: &dyn Fn(&crate::wikicode::Wikicode),
        mark: &dyn Fn(),
    ) {
        write(&[&self.as_str()]);
    }

    /// For downcasting trait objects.
    fn as_any(&self) -> &dyn Any;
}

impl Display for dyn Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl dyn Node {
    /// Downcast to a specific type if possible.
    pub fn downcast_ref<T: Node>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    /// Downcast to a specific type if possible, consuming self.
    pub fn downcast<T: Node>(self: Box<Self>) -> Result<Box<T>, Box<dyn Node>> {
        self.as_any().downcast::<T>().map_err(|e| e)
    }
}
