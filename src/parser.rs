const XML_RPC_URL: &str = "http://localhost:8000/RPC2";

pub fn call_parser(input: &str) -> anyhow::Result<String> {
    let mut client = xml_rpc::Client::new().unwrap();
    client.call(&xml_rpc::Url::parse(XML_RPC_URL)?, "parse", &[input])
        .map_err(|e| anyhow::anyhow!("XML-RPC call failed: {}", e))
        .and_then(|response| {
            Ok(response.map_err(|e| anyhow::anyhow!("XML-RPC response error: {:?}", e))?)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_parser() {
        let input = "{{PortGraph|name=TestGraph}}";
        let result = call_parser(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }
}