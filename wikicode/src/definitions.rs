// Contains data about certain markup, like HTML tags and external links.

static URI_SCHEMES: &[(&str, bool)] = &[
    ("bitcoin", false),
    ("ftp", true),
    ("ftps", true),
    ("geo", false),
    ("git", true),
    ("gopher", true),
    ("http", true),
    ("https", true),
    ("irc", true),
    ("ircs", true),
    ("magnet", false),
    ("mailto", false),
    ("mms", true),
    ("news", false),
    ("nntp", true),
    ("redis", true),
    ("sftp", true),
    ("sip", false),
    ("sips", false),
    ("sms", false),
    ("ssh", true),
    ("svn", true),
    ("tel", false),
    ("telnet", true),
    ("urn", false),
    ("worldwind", true),
    ("xmpp", false),
];

static PARSER_BLACKLIST: &[&str] = &[
    "categorytree",
    "ce",
    "chem",
    "gallery",
    "graph",
    "hiero",
    "imagemap",
    "inputbox",
    "math",
    "nowiki",
    "pre",
    "score",
    "section",
    "source",
    "syntaxhighlight",
    "templatedata",
    "timeline",
];

static INVISIBLE_TAGS: &[&str] = &[
    "categorytree",
    "gallery",
    "graph",
    "imagemap",
    "inputbox",
    "math",
    "score",
    "section",
    "templatedata",
    "timeline",
];

static SINGLE_ONLY: &[&str] = &["br", "wbr", "hr", "meta", "link", "img"];
static SINGLE: &[&str] = &[
    "br", "wbr", "hr", "meta", "link", "img", "li", "dt", "dd", "th", "td", "tr",
];

/// Return the HTML tag associated with the given wiki-markup.
/// Panics if the markup is unknown.
pub fn get_html_tag(markup: &str) -> &'static str {
    match markup {
        "#" | "*" => "li",
        ";" => "dt",
        ":" => "dd",
        _ => panic!("Unknown markup: {}", markup),
    }
}

/// Return whether the given tag's contents should be passed to the parser.
pub fn is_parsable(tag: &str) -> bool {
    let tag_lower = tag.to_lowercase();
    !PARSER_BLACKLIST.contains(&tag_lower.as_str())
}

/// Return whether or not the given tag contains visible text.
pub fn is_visible(tag: &str) -> bool {
    let tag_lower = tag.to_lowercase();
    !INVISIBLE_TAGS.contains(&tag_lower.as_str())
}

/// Return whether the given tag can exist without a close tag.
pub fn is_single(tag: &str) -> bool {
    let tag_lower = tag.to_lowercase();
    SINGLE.contains(&tag_lower.as_str())
}

/// Return whether the given tag must exist without a close tag.
pub fn is_single_only(tag: &str) -> bool {
    let tag_lower = tag.to_lowercase();
    SINGLE_ONLY.contains(&tag_lower.as_str())
}

/// Return whether *scheme* is valid for external links.
/// If `slashes` is true, any known scheme is accepted;
/// if false, only schemes that map to false are accepted.
pub fn is_scheme(scheme: &str, slashes: bool) -> bool {
    let scheme_lower = scheme.to_lowercase();
    for &(s, flag) in URI_SCHEMES.iter() {
        if s == scheme_lower {
            return if slashes { true } else { !flag };
        }
    }
    false
}
