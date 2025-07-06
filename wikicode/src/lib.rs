mod builder;
mod contexts;
mod definitions;
pub mod nodes;
mod tokenizer;
mod tokens;

pub fn tokenize(input: String) -> Result<Vec<tokens::Token>, tokenizer::TokenizerError> {
    let mut tokenizer = tokenizer::Tokenizer::new();
    tokenizer.tokenize(input, 0, false)
}

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Tokenization error: {0}")]
    TokenizationError(#[from] tokenizer::TokenizerError),
    #[error("Builder error: {0}")]
    BuilderError(#[from] builder::BuilderError),
}

pub fn parse(input: &str) -> Result<nodes::Wikicode, ParserError> {
    let mut tokenizer = tokenizer::Tokenizer::new();
    let tokens = tokenizer.tokenize(input.to_string(), 0, false)?;
    let mut builder = builder::Builder::new();
    Ok(builder.build(tokens)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_durability() {
        let inputs = [
            "== Heading ==\n\nThis is a paragraph with a [[link]].",
            "{{Template|param1=value1|param2=value2}}",
            "[[Category:Test]]",
            "<ref>Reference text</ref>",
            "<nowiki>Some unprocessed text</nowiki>",
            "This is a <b>bold</b> text and this is <i>italic</i>.",
        ];
        for input in inputs {
            let result = parse(input);
            assert!(result.is_ok(), "Failed to parse input: {}", input);
            let wikicode = result.unwrap();
            assert!(!wikicode.nodes.is_empty(), "Parsed wikicode is empty for input: {}", input);
        }
    }
}
