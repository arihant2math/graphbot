use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::schema::LocalizableString;

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
    pub data: Vec<Vec<Value>>,
}
