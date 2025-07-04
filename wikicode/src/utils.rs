use crate::{nodes::Node, wikicode::Wikicode};

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

macro_rules! into_parse_input {
    ($t:ty, $variant:ident) => {
        impl From<$t> for ParseInput<'_> {
            fn from(value: $t) -> Self {
                ParseInput::$variant(value)
            }
        }
    };
}

into_parse_input!(Wikicode, Wikicode);
into_parse_input!(Box<dyn Node>, Node);
into_parse_input!(&'static str, Str);
into_parse_input!(i64, Int);

/// Defaults:
/// - `context`: 0
/// - `skip_style_tags`: false
pub fn parse_anything<'a, T>(value: T, context: usize, skip_style_tags: bool) -> Wikicode where T: Into<ParseInput<'a>> {
    todo!()
    // match value.into() {
    //     ParseInput::Wikicode(wc) => wc,
    //     ParseInput::Node(node) => Wikicode::new(vec![node]),
    //     ParseInput::Str(s) => Parser::new().parse(s, context, skip_style_tags),
    //     ParseInput::Bytes(b) => {
    //         let s = std::str::from_utf8(b).expect("Invalid UTF-8");
    //         Parser::new().parse(s, context, skip_style_tags)
    //     }
    //     ParseInput::Int(i) => Parser::new().parse(&i.to_string(), context, skip_style_tags),
    //     ParseInput::None => Wikicode::new(Vec::new()),
    //     ParseInput::Reader(reader) => {
    //         use std::io::Read;
    //         let mut buf = String::new();
    //         reader.read_to_string(&mut buf).expect("Failed to read");
    //         parse_anything(ParseInput::Str(&buf), context, skip_style_tags)
    //     }
    //     ParseInput::Iterable(items) => {
    //         let mut nodelist = Vec::new();
    //         for item in items {
    //             let wc = parse_anything(item, context, skip_style_tags);
    //             nodelist.extend(wc.nodes());
    //         }
    //         Wikicode::new(nodelist)
    //     }
    // }
}
