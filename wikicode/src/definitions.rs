use phf::{phf_map, phf_set};

pub static URI_SCHEMES: phf::Map<&'static str, bool> = phf_map! {
    "bitcoin" => false,
    "ftp" => true,
    "ftps" => true,
    "geo" => false,
    "git" => true,
    "gopher" => true,
    "http" => true,
    "https" => true,
    "irc" => true,
    "ircs" => true,
    "magnet" => false,
    "mailto" => false,
    "mms" => true,
    "news" => false,
    "nntp" => true,
    "redis" => true,
    "sftp" => true,
    "sip" => false,
    "sips" => false,
    "sms" => false,
    "ssh" => true,
    "svn" => true,
    "tel" => false,
    "telnet" => true,
    "urn" => false,
    "worldwind" => true,
    "xmpp" => false,
};

pub static PARSER_BLACKLIST: phf::Set<&'static str> = phf_set! {
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
};

pub static INVISIBLE_TAGS: phf::Set<&'static str> = phf_set! {
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
};

pub static SINGLE_ONLY: phf::Set<&'static str> = phf_set! {
    "br", "wbr", "hr", "meta", "link", "img"
};

pub static SINGLE: phf::Set<&'static str> = phf_set! {
    "br", "wbr", "hr", "meta", "link", "img",
    "li", "dt", "dd", "th", "td", "tr"
};

pub static MARKUP_TO_HTML: phf::Map<&'static str, &'static str> = phf_map! {
    "#" => "li",
    "*" => "li",
    ";" => "dt",
    ":" => "dd",
};

pub fn get_html_tag(markup: &str) -> Option<&'static str> {
    MARKUP_TO_HTML.get(markup).copied()
}

pub fn is_parsable(tag: &str) -> bool {
    !PARSER_BLACKLIST.contains(&tag.to_ascii_lowercase()[..])
}

pub fn is_visible(tag: &str) -> bool {
    !INVISIBLE_TAGS.contains(&tag.to_ascii_lowercase()[..])
}

pub fn is_single(tag: &str) -> bool {
    SINGLE.contains(&tag.to_ascii_lowercase()[..])
}

pub fn is_single_only(tag: &str) -> bool {
    SINGLE_ONLY.contains(&tag.to_ascii_lowercase()[..])
}

pub fn is_scheme(scheme: &str, slashes: bool) -> bool {
    let scheme_lc = scheme.to_ascii_lowercase();
    match URI_SCHEMES.get(&scheme_lc[..]) {
        Some(&val) => {
            if slashes {
                true
            } else {
                !val
            }
        }
        None => false,
    }
}
