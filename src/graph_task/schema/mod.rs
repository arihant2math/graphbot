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
