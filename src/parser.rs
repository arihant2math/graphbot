use crate::config::Config;

pub fn call_parser(input: &str, config: &Config) -> anyhow::Result<String> {
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
    use super::*;

    #[test]
    fn test_call_parser() {
        // TODO: Fix
        // let input = "{{PortGraph|name=TestGraph}}";
        // let result = call_parser(input);
        // assert!(result.is_ok());
        // let output = result.unwrap();
        // assert!(!output.is_empty());
    }
}
