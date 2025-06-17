pub mod chart;
pub mod tab;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalizableString(pub HashMap<String, String>);

impl LocalizableString {
    pub fn en(s: String) -> Self {
        let mut map = HashMap::new();
        map.insert("en".to_string(), s);
        Self(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaWikiCategory {
    pub name: String
}

impl MediaWikiCategory {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn chart() -> Self {
        Self::new("Charts".to_string())
    }

    pub fn tab() -> Self {
        Self::new("Tabular data".to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaWikiCategories(pub Vec<MediaWikiCategory>);

impl MediaWikiCategories {
    pub fn new(categories: Vec<MediaWikiCategory>) -> Self {
        Self(categories)
    }

    pub fn chart() -> Self {
        Self(vec![MediaWikiCategory::chart()])
    }

    pub fn tab() -> Self {
        Self(vec![MediaWikiCategory::tab()])
    }
}
