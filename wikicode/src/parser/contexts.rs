pub const TEMPLATE_NAME: u64 = 1 << 0;
pub const TEMPLATE_PARAM_KEY: u64 = 1 << 1;
pub const TEMPLATE_PARAM_VALUE: u64 = 1 << 2;
pub const TEMPLATE: u64 = TEMPLATE_NAME | TEMPLATE_PARAM_KEY | TEMPLATE_PARAM_VALUE;

pub const ARGUMENT_NAME: u64 = 1 << 3;
pub const ARGUMENT_DEFAULT: u64 = 1 << 4;
pub const ARGUMENT: u64 = ARGUMENT_NAME | ARGUMENT_DEFAULT;

pub const WIKILINK_TITLE: u64 = 1 << 5;
pub const WIKILINK_TEXT: u64 = 1 << 6;
pub const WIKILINK: u64 = WIKILINK_TITLE | WIKILINK_TEXT;

pub const EXT_LINK_URI: u64 = 1 << 7;
pub const EXT_LINK_TITLE: u64 = 1 << 8;
pub const EXT_LINK: u64 = EXT_LINK_URI | EXT_LINK_TITLE;

pub const HEADING_LEVEL_1: u64 = 1 << 9;
pub const HEADING_LEVEL_2: u64 = 1 << 10;
pub const HEADING_LEVEL_3: u64 = 1 << 11;
pub const HEADING_LEVEL_4: u64 = 1 << 12;
pub const HEADING_LEVEL_5: u64 = 1 << 13;
pub const HEADING_LEVEL_6: u64 = 1 << 14;
pub const HEADING: u64 = HEADING_LEVEL_1
    | HEADING_LEVEL_2
    | HEADING_LEVEL_3
    | HEADING_LEVEL_4
    | HEADING_LEVEL_5
    | HEADING_LEVEL_6;

pub const TAG_OPEN: u64 = 1 << 15;
pub const TAG_ATTR: u64 = 1 << 16;
pub const TAG_BODY: u64 = 1 << 17;
pub const TAG_CLOSE: u64 = 1 << 18;
pub const TAG: u64 = TAG_OPEN | TAG_ATTR | TAG_BODY | TAG_CLOSE;

pub const STYLE_ITALICS: u64 = 1 << 19;
pub const STYLE_BOLD: u64 = 1 << 20;
pub const STYLE_PASS_AGAIN: u64 = 1 << 21;
pub const STYLE_SECOND_PASS: u64 = 1 << 22;
pub const STYLE: u64 = STYLE_ITALICS | STYLE_BOLD | STYLE_PASS_AGAIN | STYLE_SECOND_PASS;

pub const DL_TERM: u64 = 1 << 23;

pub const HAS_TEXT: u64 = 1 << 24;
pub const FAIL_ON_TEXT: u64 = 1 << 25;
pub const FAIL_NEXT: u64 = 1 << 26;
pub const FAIL_ON_LBRACE: u64 = 1 << 27;
pub const FAIL_ON_RBRACE: u64 = 1 << 28;
pub const FAIL_ON_EQUALS: u64 = 1 << 29;
pub const HAS_TEMPLATE: u64 = 1 << 30;
pub const SAFETY_CHECK: u64 = HAS_TEXT
    | FAIL_ON_TEXT
    | FAIL_NEXT
    | FAIL_ON_LBRACE
    | FAIL_ON_RBRACE
    | FAIL_ON_EQUALS
    | HAS_TEMPLATE;

pub const TABLE_OPEN: u64 = 1 << 31;
pub const TABLE_CELL_OPEN: u64 = 1 << 32;
pub const TABLE_CELL_STYLE: u64 = 1 << 33;
pub const TABLE_ROW_OPEN: u64 = 1 << 34;
pub const TABLE_TD_LINE: u64 = 1 << 35;
pub const TABLE_TH_LINE: u64 = 1 << 36;
pub const TABLE_CELL_LINE_CONTEXTS: u64 = TABLE_TD_LINE | TABLE_TH_LINE | TABLE_CELL_STYLE;
pub const TABLE: u64 = TABLE_OPEN
    | TABLE_CELL_OPEN
    | TABLE_CELL_STYLE
    | TABLE_ROW_OPEN
    | TABLE_TD_LINE
    | TABLE_TH_LINE;

pub const HTML_ENTITY: u64 = 1 << 37;

// Global contexts:
pub const GL_HEADING: u64 = 1 << 0;

// Aggregate contexts:
pub const FAIL: u64 =
    TEMPLATE | ARGUMENT | WIKILINK | EXT_LINK_TITLE | HEADING | TAG | STYLE | TABLE;

pub const UNSAFE: u64 = TEMPLATE_NAME
    | WIKILINK_TITLE
    | EXT_LINK_TITLE
    | TEMPLATE_PARAM_KEY
    | ARGUMENT_NAME
    | TAG_CLOSE;

pub const DOUBLE: u64 = TEMPLATE_PARAM_KEY | TAG_CLOSE | TABLE_ROW_OPEN;

pub const NO_WIKILINKS: u64 = TEMPLATE_NAME | ARGUMENT_NAME | WIKILINK_TITLE | EXT_LINK_URI;

pub const NO_EXT_LINKS: u64 = TEMPLATE_NAME | ARGUMENT_NAME | WIKILINK_TITLE | EXT_LINK;

// Function to describe the context for debugging
pub fn describe(context: u64) -> String {
    let mut flags = Vec::new();
    let contexts = [
        ("TEMPLATE_NAME", TEMPLATE_NAME),
        ("TEMPLATE_PARAM_KEY", TEMPLATE_PARAM_KEY),
        ("TEMPLATE_PARAM_VALUE", TEMPLATE_PARAM_VALUE),
        ("ARGUMENT_NAME", ARGUMENT_NAME),
        ("ARGUMENT_DEFAULT", ARGUMENT_DEFAULT),
        ("WIKILINK_TITLE", WIKILINK_TITLE),
        ("WIKILINK_TEXT", WIKILINK_TEXT),
        ("EXT_LINK_URI", EXT_LINK_URI),
        ("EXT_LINK_TITLE", EXT_LINK_TITLE),
        ("HEADING_LEVEL_1", HEADING_LEVEL_1),
        ("HEADING_LEVEL_2", HEADING_LEVEL_2),
        ("HEADING_LEVEL_3", HEADING_LEVEL_3),
        ("HEADING_LEVEL_4", HEADING_LEVEL_4),
        ("HEADING_LEVEL_5", HEADING_LEVEL_5),
        ("HEADING_LEVEL_6", HEADING_LEVEL_6),
        ("TAG_OPEN", TAG_OPEN),
        ("TAG_ATTR", TAG_ATTR),
        ("TAG_BODY", TAG_BODY),
        ("TAG_CLOSE", TAG_CLOSE),
        ("STYLE_ITALICS", STYLE_ITALICS),
        ("STYLE_BOLD", STYLE_BOLD),
        ("STYLE_PASS_AGAIN", STYLE_PASS_AGAIN),
        ("STYLE_SECOND_PASS", STYLE_SECOND_PASS),
        ("DL_TERM", DL_TERM),
        ("HAS_TEXT", HAS_TEXT),
        ("FAIL_ON_TEXT", FAIL_ON_TEXT),
        ("FAIL_NEXT", FAIL_NEXT),
        ("FAIL_ON_LBRACE", FAIL_ON_LBRACE),
        ("FAIL_ON_RBRACE", FAIL_ON_RBRACE),
        ("FAIL_ON_EQUALS", FAIL_ON_EQUALS),
        ("HAS_TEMPLATE", HAS_TEMPLATE),
        ("TABLE_OPEN", TABLE_OPEN),
        ("TABLE_CELL_OPEN", TABLE_CELL_OPEN),
        ("TABLE_CELL_STYLE", TABLE_CELL_STYLE),
        ("TABLE_ROW_OPEN", TABLE_ROW_OPEN),
        ("TABLE_TD_LINE", TABLE_TD_LINE),
        ("TABLE_TH_LINE", TABLE_TH_LINE),
        ("HTML_ENTITY", HTML_ENTITY),
    ];

    for &(name, value) in &contexts {
        if context & value != 0 {
            flags.push(name);
        }
    }

    flags.join("|")
}
