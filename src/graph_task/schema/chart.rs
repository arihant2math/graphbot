use serde::{Deserialize, Serialize};

use crate::graph_task::schema::{LocalizableString, MediaWikiCategories};

fn default_axis_format() -> String {
    "None".to_string()
}

fn is_default_axis_format(format: &str) -> bool {
    format == "None"
}

fn default_version() -> u64 {
    1
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    #[default]
    Line,
    Pie,
    Bar,
    Area,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub license: String,
    #[serde(default = "default_version")]
    pub version: u64,
    pub r#type: ChartType,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub mediawikiCategories: Option<MediaWikiCategories>
}

impl Default for Chart {
    fn default() -> Self {
        Chart {
            license: <_>::default(),
            version: default_version(),
            r#type: <_>::default(),
            source: <_>::default(),
            x_axis: None,
            y_axis: None,
            title: None,
            mediawikiCategories: Some(MediaWikiCategories::chart()),
        }
    }
}
