//! CLASSIFICATION — what kind of business, product or service a thing is, said on purpose by its account
//! (CommandAGI's docs/formats.md § classification). A `company()`, a company's `product()`, a `products::product()`
//! and a storefront's offer take two attributes:
//!
//! ```text
//! .codes(["naics:541511", "unspsc:43232100"])   standard codes as scheme:code
//! .tags(["robotics", "3d printing"])             free words: lower case, trimmed, single spaces, at most 40 characters
//! ```
//!
//! Business schemes classify what a company does (naics, nace, isic, sic, anzsic, jsic); product schemes what is sold
//! (unspsc, gs1, hs). A shorter code of a scheme is the group of the longer codes it starts (`naics:54` holds
//! `naics:541511`). The same grammar as the TypeScript SDK's `classification.ts`, the Python SDK's `classification.py`
//! and CommandAGI's `packages/domain/world/classification.js`.
//!
//! ```
//! use commandagi::design::classification::*;
//! use commandagi::design::Json;
//!
//! let codes = Json::Arr(vec![Json::Str("naics:54".into())]);
//! assert_eq!(classification_problem(Some(&codes), None, Some(PRODUCT_SCHEMES)).unwrap(),
//!            "codes: \"naics:54\" is a business code; here a code is of unspsc, gs1, hs");
//! assert!(matches_prefix("naics:541511", "naics:54"));
//! ```

use super::Json;

/// What a code after its scheme may be. The crate has no regex engine, so each grammar is read by hand.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Grammar {
    /// From `min` to `max` digits.
    Digits(usize, usize),
    /// Pairs of digits: 2, 4 … up to `max` digits.
    DigitPairs(usize),
    /// A section letter from `A` to the letter, or `min` to `max` digits.
    LetterOrDigits(char, usize, usize),
    /// NACE: a section letter A-U, or a division, group or class (`28`, `28.4`, `28.41`).
    Nace,
}

/// One scheme of codes.
#[derive(Clone, Copy, Debug)]
pub struct Scheme {
    pub id: &'static str,
    /// `business` or `product`.
    pub kind: &'static str,
    pub name: &'static str,
    pub pattern: Grammar,
    pub example: &'static str,
    pub grammar: &'static str,
}

pub const SCHEMES: &[Scheme] = &[
    Scheme {
        id: "naics",
        kind: "business",
        name: "NAICS",
        pattern: Grammar::Digits(2, 6),
        example: "naics:541511",
        grammar: "2 to 6 digits",
    },
    Scheme {
        id: "nace",
        kind: "business",
        name: "NACE",
        pattern: Grammar::Nace,
        example: "nace:28.41",
        grammar: "a section letter A-U, or a division, group or class (28, 28.4, 28.41)",
    },
    Scheme {
        id: "isic",
        kind: "business",
        name: "ISIC",
        pattern: Grammar::LetterOrDigits('U', 2, 4),
        example: "isic:2822",
        grammar: "a section letter A-U, or 2 to 4 digits",
    },
    Scheme {
        id: "sic",
        kind: "business",
        name: "SIC",
        pattern: Grammar::Digits(2, 4),
        example: "sic:3569",
        grammar: "2 to 4 digits",
    },
    Scheme {
        id: "anzsic",
        kind: "business",
        name: "ANZSIC",
        pattern: Grammar::LetterOrDigits('S', 2, 4),
        example: "anzsic:2469",
        grammar: "a division letter A-S, or 2 to 4 digits",
    },
    Scheme {
        id: "jsic",
        kind: "business",
        name: "JSIC",
        pattern: Grammar::LetterOrDigits('T', 2, 4),
        example: "jsic:2694",
        grammar: "a division letter A-T, or 2 to 4 digits",
    },
    Scheme {
        id: "unspsc",
        kind: "product",
        name: "UNSPSC",
        pattern: Grammar::DigitPairs(8),
        example: "unspsc:43232100",
        grammar: "2, 4, 6 or 8 digits",
    },
    Scheme {
        id: "gs1",
        kind: "product",
        name: "GS1 GPC",
        pattern: Grammar::Digits(8, 8),
        example: "gs1:10001234",
        grammar: "8 digits (a GPC brick or a level above it)",
    },
    Scheme {
        id: "hs",
        kind: "product",
        name: "HS",
        pattern: Grammar::DigitPairs(10),
        example: "hs:847989",
        grammar: "2, 4, 6, 8 or 10 digits",
    },
];

pub const SCHEME_NAMES: &[&str] = &[
    "naics", "nace", "isic", "sic", "anzsic", "jsic", "unspsc", "gs1", "hs",
];
pub const BUSINESS_SCHEMES: &[&str] = &["naics", "nace", "isic", "sic", "anzsic", "jsic"];
pub const PRODUCT_SCHEMES: &[&str] = &["unspsc", "gs1", "hs"];
pub const MAX_CODES: usize = 20;
pub const MAX_TAGS: usize = 20;
pub const MAX_TAG_LENGTH: usize = 40;

/// The scheme of that id.
pub fn scheme(id: &str) -> Option<&'static Scheme> {
    SCHEMES.iter().find(|s| s.id == id)
}

fn digits(s: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit())
}

fn letter(s: &str, last: char) -> bool {
    let mut chars = s.chars();
    matches!((chars.next(), chars.next()), (Some(c), None) if ('A'..=last).contains(&c))
}

impl Grammar {
    /// Whether `s` (a code after its scheme) is of this grammar.
    pub fn matches(self, s: &str) -> bool {
        match self {
            Grammar::Digits(min, max) => digits(s, min, max),
            Grammar::DigitPairs(max) => s.len() % 2 == 0 && digits(s, 2, max),
            Grammar::LetterOrDigits(last, min, max) => letter(s, last) || digits(s, min, max),
            Grammar::Nace => {
                letter(s, 'U')
                    || match s.split_once('.') {
                        None => digits(s, 2, 2),
                        Some((division, rest)) => digits(division, 2, 2) && digits(rest, 1, 2),
                    }
            }
        }
    }
}

/// A value as the TypeScript SDK says it: a text quoted, anything else as JavaScript's `String(v)`.
fn said(v: &Json) -> String {
    match v {
        Json::Str(_) => v.text(),
        Json::Arr(items) => items
            .iter()
            .map(|x| {
                if let Json::Str(s) = x {
                    s.clone()
                } else if *x == Json::Null {
                    String::new()
                } else {
                    said(x)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        Json::Obj(_) => "[object Object]".into(),
        other => other.text(),
    }
}

fn quoted(s: &str) -> String {
    Json::Str(s.to_string()).text()
}

/// Why `raw` is not a code, or `None` when it is one.
pub fn code_problem(raw: &Json) -> Option<String> {
    let Json::Str(code) = raw else {
        return Some(format!(
            "a code is a \"scheme:code\" text, not {}",
            said(raw)
        ));
    };
    let Some((id, rest)) = code.split_once(':').filter(|(id, _)| !id.is_empty()) else {
        return Some(format!(
            "{} is not a code: write it as scheme:code ({})",
            quoted(code),
            SCHEMES[0].example
        ));
    };
    let Some(s) = scheme(id) else {
        return Some(format!(
            "{}: \"{id}\" is not a known scheme ({})",
            quoted(code),
            SCHEME_NAMES.join(", ")
        ));
    };
    if !s.pattern.matches(rest) {
        return Some(format!(
            "{}: a {} code is {} ({})",
            quoted(code),
            s.name,
            s.grammar,
            s.example
        ));
    }
    None
}

/// A tag as a file writes it: trimmed, lower case, single spaces.
pub fn normalize_tag(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Why `raw` is not a tag, or `None`.
pub fn tag_problem(raw: &Json) -> Option<String> {
    let Json::Str(tag) = raw else {
        return Some(format!("a tag is a text, not {}", said(raw)));
    };
    let norm = normalize_tag(tag);
    if norm.is_empty() {
        return Some("a tag is not empty".into());
    }
    // JavaScript's length: UTF-16 code units.
    if norm.encode_utf16().count() > MAX_TAG_LENGTH {
        return Some(format!(
            "{}: a tag is at most {MAX_TAG_LENGTH} characters",
            quoted(tag)
        ));
    }
    if &norm != tag {
        return Some(format!(
            "{}: write the tag as {} (lower case, trimmed, single spaces)",
            quoted(tag),
            quoted(&norm)
        ));
    }
    None
}

/// The first problem of a thing's `codes` and `tags` (`None` is absent), or `None`; `schemes` narrows the schemes allowed.
pub fn classification_problem(
    codes: Option<&Json>,
    tags: Option<&Json>,
    schemes: Option<&[&str]>,
) -> Option<String> {
    if let Some(codes) = codes {
        let Json::Arr(list) = codes else {
            return Some(format!(
                "codes is a list of \"scheme:code\" texts, not {}",
                said(codes)
            ));
        };
        if list.len() > MAX_CODES {
            return Some(format!(
                "codes holds at most {MAX_CODES} codes, not {}",
                list.len()
            ));
        }
        let mut seen: Vec<&str> = Vec::new();
        for c in list {
            if let Some(p) = code_problem(c) {
                return Some(format!("codes: {p}"));
            }
            let Json::Str(code) = c else {
                unreachable!("a code is a text")
            };
            let id = &code[..code.find(':').unwrap_or(0)];
            if let Some(allowed) = schemes.filter(|allowed| !allowed.contains(&id)) {
                let kind = scheme(id).map_or("", |s| s.kind);
                return Some(format!(
                    "codes: {} is a {kind} code; here a code is of {}",
                    quoted(code),
                    allowed.join(", ")
                ));
            }
            if seen.contains(&code.as_str()) {
                return Some(format!("codes: {} is written twice", quoted(code)));
            }
            seen.push(code);
        }
    }
    if let Some(tags) = tags {
        let Json::Arr(list) = tags else {
            return Some(format!("tags is a list of words, not {}", said(tags)));
        };
        if list.len() > MAX_TAGS {
            return Some(format!(
                "tags holds at most {MAX_TAGS} tags, not {}",
                list.len()
            ));
        }
        let mut seen: Vec<&str> = Vec::new();
        for t in list {
            if let Some(p) = tag_problem(t) {
                return Some(format!("tags: {p}"));
            }
            let Json::Str(tag) = t else {
                unreachable!("a tag is a text")
            };
            if seen.contains(&tag.as_str()) {
                return Some(format!("tags: {} is written twice", quoted(tag)));
            }
            seen.push(tag);
        }
    }
    None
}

/// Whether `code` is under `prefix` (`naics:54` holds `naics:541511`; `naics` holds every NAICS code).
pub fn matches_prefix(code: &str, prefix: &str) -> bool {
    if code_problem(&Json::Str(code.to_string())).is_some() {
        return false;
    }
    let (id, digits) = code.split_once(':').unwrap_or((code, ""));
    match prefix.split_once(':') {
        None => id == prefix,
        Some((p, "")) => id == p,
        Some((p, start)) => id == p && digits.starts_with(start),
    }
}
