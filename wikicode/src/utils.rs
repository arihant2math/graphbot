use crate::{nodes::Node, parser::Parser, wikicode::Wikicode};

pub enum ParseInput<'a> {
    Wikicode(Wikicode),
    Node(Box<dyn Node>),
    Str(&'a str),
    Bytes(&'a [u8]),
    Int(i64),
    None,
    Reader(&'a mut dyn std::io::Read),
    Iterable(Vec<ParseInput<'a>>),
}

pub fn parse_anything(value: ParseInput, context: usize, skip_style_tags: bool) -> Wikicode {
    match value {
        ParseInput::Wikicode(wc) => wc,
        ParseInput::Node(node) => Wikicode::new(vec![node]),
        ParseInput::Str(s) => Parser::new().parse(s, context, skip_style_tags),
        ParseInput::Bytes(b) => {
            let s = std::str::from_utf8(b).expect("Invalid UTF-8");
            Parser::new().parse(s, context, skip_style_tags)
        }
        ParseInput::Int(i) => Parser::new().parse(&i.to_string(), context, skip_style_tags),
        ParseInput::None => Wikicode::new(Vec::new()),
        ParseInput::Reader(reader) => {
            use std::io::Read;
            let mut buf = String::new();
            reader.read_to_string(&mut buf).expect("Failed to read");
            parse_anything(ParseInput::Str(&buf), context, skip_style_tags)
        }
        ParseInput::Iterable(items) => {
            let mut nodelist = Vec::new();
            for item in items {
                let wc = parse_anything(item, context, skip_style_tags);
                nodelist.extend(wc.nodes());
            }
            Wikicode::new(nodelist)
        }
    }
}
