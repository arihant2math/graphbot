use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevInfo {
    pub id: u64,
    pub page_title: String,
}

impl RevInfo {
    pub fn new(id: u64, page_title: String) -> Self {
        Self { id, page_title }
    }
}

impl Hash for RevInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for RevInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for RevInfo {}
