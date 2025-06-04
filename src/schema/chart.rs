use crate::schema::LocalizableString;
use serde::{Deserialize, Serialize};

fn default_axis_format() -> String {
    "None".to_string()
}

fn is_default_axis_format(format: &str) -> bool {
    format == "None"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axis {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub title: Option<LocalizableString>,
    #[serde(skip_serializing_if = "is_default_axis_format")]
    #[serde(default = "default_axis_format")]
    pub format: String,
}

impl Default for Axis {
    fn default() -> Self {
        Axis {
            title: None,
            format: "None".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub license: String,
    pub version: u64,
    pub r#type: String,
    #[serde(rename = "xAxis")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub x_axis: Option<Axis>,
    #[serde(rename = "yAxis")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub y_axis: Option<Axis>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub title: Option<LocalizableString>,
}
