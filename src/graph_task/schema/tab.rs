use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::graph_task::schema::{LocalizableString, MediaWikiCategories};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub title: Option<LocalizableString>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub fields: Vec<Field>,
}

impl From<Vec<Field>> for Schema {
    fn from(fields: Vec<Field>) -> Self {
        Schema { fields }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tab {
    pub license: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub description: Option<LocalizableString>,
    pub schema: Schema,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub sources: Option<String>,
    pub data: Vec<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub mediawikiCategories: Option<MediaWikiCategories>,
}

impl Default for Tab {
    fn default() -> Self {
        Tab {
            license: "CC-BY-SA-4.0".to_string(),
            description: None,
            schema: Schema::from(vec![]),
            data: vec![],
            sources: None,
            mediawikiCategories: Some(MediaWikiCategories::tab()),
        }
    }
}
