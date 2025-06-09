use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub params: HashMap<String, Option<String>>,
    pub wikitext: String,
}

#[derive(Serialize, Deserialize)]
pub struct OutRoot {
    pub templates: Vec<Template>,
    pub tags: Vec<String>,
    pub elapsed: f64,
}

pub fn call_parser(input: &str, config: &Config) -> anyhow::Result<OutRoot> {
    let mut client = xml_rpc::Client::new().unwrap();
    client
        .call(
            &xml_rpc::Url::parse(&format!(
                "http://localhost:{}/{}",
                config.rpc.port, config.rpc.path
            ))?,
            "parse",
            [input],
        )
        .map_err(|e| anyhow::anyhow!("XML-RPC call failed: {}", e))
        .and_then(|response| response.map_err(|e| anyhow::anyhow!("XML-RPC response error: {e:?}")))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_call_parser() {
        // TODO: Fix
        // let input = "{{PortGraph|name=TestGraph}}";
        // let result = call_parser(input, Config::default());
        // assert!(result.is_ok());
        // let output = result.unwrap();
        // assert!(!output.is_empty());
    }
}
