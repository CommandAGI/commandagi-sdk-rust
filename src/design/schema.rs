//! THE SCHEMA OF AN ELEMENT'S PROPS, AND ITS CHECK — the TypeScript SDK's `schema.ts`, in Rust. A document's elements
//! (`extrude().distance(10)`) take fields whose JSON Schema is generated from the types that declare them
//! (`threed_schema.json`, scripts/workbench/schemas.mjs in the CommandAGI repository). The reader checks each element
//! against it and refuses a wrong value with the sentence the TypeScript reader, the op panel and the agent's tool give:
//! `distance: must be greater than 0 (got -1)`.
//!
//! The subset: type, enum, const, properties, required, additionalProperties, items, prefixItems, minItems, maxItems,
//! minimum, maximum, exclusiveMinimum, exclusiveMaximum, anyOf, oneOf, `$ref` to `#/$defs/<name>`, and the two
//! `pattern`s the generated schemas use (`\S`, `^[A-Za-z0-9_.-]+$`, `^(?:[A-Z]{2}|unknown)$`, `^[A-Z]{3}$`,
//! `^\d{4}-\d{2}-\d{2}$`). The crate has no regex engine: any other pattern
//! is refused by name, never skipped.

use super::json::Json;

const REF: &str = "#/$defs/";

/// What way a value breaks its schema (the TypeScript `IssueCode`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueCode {
    Required,
    Type,
    Enum,
    Const,
    Minimum,
    Maximum,
    ExclusiveMinimum,
    ExclusiveMaximum,
    Integer,
    MinItems,
    MaxItems,
    Additional,
    Pattern,
    AnyOf,
}

/// One way a value breaks its schema: where (`distance`, `edges/0/kind`; "" is the value itself), what the schema asks
/// for, what the value had.
#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub path: String,
    pub code: IssueCode,
    pub expected: Option<Json>,
    pub actual: Option<Json>,
}

impl Issue {
    fn new(path: &str, code: IssueCode, expected: Option<Json>, actual: Option<Json>) -> Issue {
        Issue { path: path.to_string(), code, expected, actual }
    }
}

/// The definition a `$ref` names, with the referring schema's own keywords over it.
pub fn resolve(schema: &Json, defs: &Json) -> Result<Json, String> {
    let mut s = schema.clone();
    let mut guard = 0;
    while let Some(r) = s.get("$ref").and_then(Json::as_str).map(str::to_string) {
        guard += 1;
        if guard > 32 {
            return Err(format!("{r}: a reference cycle"));
        }
        let name = r.strip_prefix(REF).ok_or_else(|| format!("{r}: only #/$defs/<name> references are read"))?;
        let target = defs.get(name).ok_or_else(|| format!("{r}: no such definition"))?;
        let mut merged = target.clone();
        if let Json::Obj(own) = &s {
            for (k, v) in own {
                if k != "$ref" {
                    merged.set(k, v.clone());
                }
            }
        }
        s = merged;
    }
    Ok(s)
}

/// `{ "$file": "Part.assets/f3/positions.f64", … }`: data kept in a file beside the document.
pub fn is_file_ref(v: &Json) -> bool {
    matches!(v.get("$file"), Some(Json::Str(_)))
}

fn is_integer(n: f64) -> bool {
    n.is_finite() && n == n.trunc()
}

fn kind_of(v: &Json) -> &'static str {
    match v {
        Json::Null => "null",
        Json::Arr(_) => "array",
        Json::Num(n) if is_integer(*n) => "integer",
        Json::Num(_) => "number",
        Json::Str(_) => "string",
        Json::Bool(_) => "boolean",
        Json::Obj(_) => "object",
    }
}

fn type_matches(t: &str, v: &Json) -> bool {
    match t {
        // Large data (an array, the bytes of a data URL) may be an ordinary file the document names (docs/formats.md).
        "string" => matches!(v, Json::Str(_)) || is_file_ref(v),
        "number" => matches!(v, Json::Num(n) if n.is_finite()),
        "integer" => matches!(v, Json::Num(n) if is_integer(*n)),
        "boolean" => matches!(v, Json::Bool(_)),
        "array" => matches!(v, Json::Arr(_)) || is_file_ref(v),
        "object" => matches!(v, Json::Obj(_)),
        "null" => matches!(v, Json::Null),
        _ => false,
    }
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() { key.to_string() } else { format!("{path}/{key}") }
}

fn num(s: &Json, k: &str) -> Option<f64> {
    s.get(k).and_then(Json::as_f64)
}

/// Every way `value` breaks `schema`; empty when it fits. `defs` is the document's `$defs`.
pub fn validate(schema: &Json, value: &Json, defs: &Json, path: &str) -> Result<Vec<Issue>, String> {
    let s = resolve(schema, defs)?;
    let mut out = Vec::new();
    if let Some(Json::Arr(options)) = s.get("anyOf").or_else(|| s.get("oneOf")) {
        let mut results = Vec::new();
        for o in options {
            let r = validate(o, value, defs, path)?;
            if r.is_empty() {
                return Ok(Vec::new());
            }
            results.push(r);
        }
        // A union of objects told apart by a constant field (`type: "datum"`): the option the value names says the most.
        for (i, o) in options.iter().enumerate() {
            let r = resolve(o, defs)?;
            if let Some(Json::Obj(props)) = r.get("properties") {
                if props.iter().any(|(k, p)| p.get("const").is_some_and(|c| value.get(k) == Some(c))) {
                    return Ok(results.swap_remove(i));
                }
            }
        }
        // A union of objects told apart by what each must hold (a pick's `signature`, a selector's `select`): when the
        // value holds every required field of one option only, that option says what is wrong in it.
        if matches!(value, Json::Obj(_)) {
            let mut holds = Vec::new();
            for (i, o) in options.iter().enumerate() {
                if let Some(Json::Arr(required)) = resolve(o, defs)?.get("required") {
                    if !required.is_empty() && required.iter().all(|k| k.as_str().is_some_and(|k| value.get(k).is_some())) {
                        holds.push(i);
                    }
                }
            }
            if holds.len() == 1 {
                return Ok(results.swap_remove(holds[0]));
            }
        }
        return Ok(vec![Issue::new(path, IssueCode::AnyOf, None, Some(Json::from(kind_of(value))))]);
    }
    let types: Vec<&str> = match s.get("type") {
        Some(Json::Str(t)) => vec![t.as_str()],
        Some(Json::Arr(ts)) => ts.iter().filter_map(Json::as_str).collect(),
        _ => Vec::new(),
    };
    if !types.is_empty() && !types.iter().any(|t| type_matches(t, value)) {
        if types.contains(&"integer") && matches!(value, Json::Num(_)) {
            return Ok(vec![Issue::new(path, IssueCode::Integer, None, Some(value.clone()))]);
        }
        return Ok(vec![Issue::new(path, IssueCode::Type, Some(Json::from(types.join(" | "))), Some(Json::from(kind_of(value))))]);
    }
    if let Some(c) = s.get("const") {
        if value != c {
            out.push(Issue::new(path, IssueCode::Const, Some(c.clone()), Some(value.clone())));
        }
    }
    if let Some(Json::Arr(allowed)) = s.get("enum") {
        if !allowed.contains(value) {
            out.push(Issue::new(path, IssueCode::Enum, Some(Json::Arr(allowed.clone())), Some(value.clone())));
        }
    }
    if let (Json::Str(text), Some(Json::Str(pattern))) = (value, s.get("pattern")) {
        if !matches_pattern(pattern, text)? {
            out.push(Issue::new(path, IssueCode::Pattern, Some(Json::from(pattern.as_str())), Some(value.clone())));
        }
    }
    if let Json::Num(n) = value {
        let n = *n;
        let bounds: [(&str, IssueCode, fn(f64, f64) -> bool); 4] = [
            ("minimum", IssueCode::Minimum, |v, b| v < b),
            ("maximum", IssueCode::Maximum, |v, b| v > b),
            ("exclusiveMinimum", IssueCode::ExclusiveMinimum, |v, b| v <= b),
            ("exclusiveMaximum", IssueCode::ExclusiveMaximum, |v, b| v >= b),
        ];
        for (k, code, breaks) in bounds {
            if let Some(b) = num(&s, k) {
                if breaks(n, b) {
                    out.push(Issue::new(path, code, Some(Json::Num(b)), Some(value.clone())));
                }
            }
        }
    }
    if let Json::Arr(items) = value {
        let len = Json::Num(items.len() as f64);
        if let Some(m) = num(&s, "minItems") {
            if (items.len() as f64) < m {
                out.push(Issue::new(path, IssueCode::MinItems, Some(Json::Num(m)), Some(len.clone())));
            }
        }
        if let Some(m) = num(&s, "maxItems") {
            if (items.len() as f64) > m {
                out.push(Issue::new(path, IssueCode::MaxItems, Some(Json::Num(m)), Some(len)));
            }
        }
        for (i, item) in items.iter().enumerate() {
            let prefix = match s.get("prefixItems") {
                Some(Json::Arr(p)) => p.get(i),
                _ => None,
            };
            if let Some(item_schema) = prefix.or_else(|| s.get("items")) {
                out.extend(validate(item_schema, item, defs, &join(path, &i.to_string()))?);
            }
        }
    }
    if let Json::Obj(entries) = value {
        let props = s.get("properties");
        let additional = s.get("additionalProperties");
        if props.is_some() || s.get("required").is_some() || additional.is_some() {
            if let Some(Json::Arr(required)) = s.get("required") {
                for k in required.iter().filter_map(Json::as_str) {
                    if value.get(k).is_none() {
                        out.push(Issue::new(&join(path, k), IssueCode::Required, None, None));
                    }
                }
            }
            for (k, v) in entries {
                if let Some(p) = props.and_then(|p| p.get(k)) {
                    out.extend(validate(p, v, defs, &join(path, k))?);
                } else if additional == Some(&Json::Bool(false)) {
                    out.push(Issue::new(&join(path, k), IssueCode::Additional, None, None));
                } else if let Some(a @ Json::Obj(_)) = additional {
                    out.extend(validate(a, v, defs, &join(path, k))?);
                }
            }
        }
    }
    Ok(out)
}

/// The patterns the generated schemas use, read without a regex engine.
fn matches_pattern(pattern: &str, text: &str) -> Result<bool, String> {
    match pattern {
        r"\S" => Ok(text.chars().any(|c| !c.is_whitespace())),
        r"^[A-Za-z0-9_.-]+$" => Ok(!text.is_empty() && text.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))),
        // An ISO 3166 alpha-2 country (or "unknown": not known yet, and said so) and an ISO 4217 currency.
        r"^(?:[A-Z]{2}|unknown)$" => Ok(text == "unknown" || (text.len() == 2 && text.bytes().all(|b| b.is_ascii_uppercase()))),
        r"^[A-Z]{3}$" => Ok(text.len() == 3 && text.bytes().all(|b| b.is_ascii_uppercase())),
        // A date, YYYY-MM-DD.
        r"^\d{4}-\d{2}-\d{2}$" => {
            let b = text.as_bytes();
            Ok(b.len() == 10 && b.iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() }))
        }
        other => Err(format!("schema pattern {other} is not one this reader knows")),
    }
}

/// Whether a value can be what `s` (resolved) describes, by its JSON kind alone.
fn kind_fits(s: &Json, value: &Json) -> bool {
    match s.get("type") {
        Some(Json::Str(t)) => type_matches(t, value),
        Some(Json::Arr(ts)) => ts.is_empty() || ts.iter().filter_map(Json::as_str).any(|t| type_matches(t, value)),
        _ => true,
    }
}

/// `value` with every absent required field that declares a `default` filled with that default, at every depth (the
/// TypeScript SDK's `withDefaults`). A reader calls it before [`validate`], so a file that leaves out a defaulted field
/// reads as if it had written the default. A field with no default, and an optional field, stay absent. A value that
/// is not the shape the schema describes comes back as it was, for [`validate`] to name.
pub fn with_defaults(schema: &Json, value: &Json, defs: &Json) -> Result<Json, String> {
    let s = resolve(schema, defs)?;
    if let Some(Json::Arr(options)) = s.get("anyOf").or_else(|| s.get("oneOf")) {
        let mut named = None;
        for o in options {
            let r = resolve(o, defs)?;
            if let Some(Json::Obj(props)) = r.get("properties") {
                if props.iter().any(|(k, p)| p.get("const").is_some_and(|c| value.get(k) == Some(c))) {
                    named = Some(o);
                    break;
                }
            }
        }
        let candidates: Vec<&Json> = match named {
            Some(o) => vec![o],
            None => {
                let mut fits = Vec::new();
                for o in options {
                    if kind_fits(&resolve(o, defs)?, value) {
                        fits.push(o);
                    }
                }
                fits
            }
        };
        if candidates.len() == 1 {
            return with_defaults(candidates[0], value, defs);
        }
        for o in candidates {
            let filled = with_defaults(o, value, defs)?;
            if validate(o, &filled, defs, "")?.is_empty() {
                return Ok(filled);
            }
        }
        return Ok(value.clone());
    }
    match value {
        Json::Arr(items) => {
            let prefix = match s.get("prefixItems") {
                Some(Json::Arr(p)) => p.as_slice(),
                _ => &[],
            };
            let all = s.get("items");
            if prefix.is_empty() && all.is_none() {
                return Ok(value.clone());
            }
            let mut out = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                out.push(match prefix.get(i).or(all) {
                    Some(item_schema) => with_defaults(item_schema, item, defs)?,
                    None => item.clone(),
                });
            }
            Ok(Json::Arr(out))
        }
        Json::Obj(entries) if !is_file_ref(value) && kind_fits(&s, value) => {
            let props = s.get("properties");
            let extra = match s.get("additionalProperties") {
                Some(a @ Json::Obj(_)) => Some(a),
                _ => None,
            };
            if props.is_none() && extra.is_none() {
                return Ok(value.clone());
            }
            let mut out = Vec::with_capacity(entries.len());
            for (k, v) in entries {
                out.push((k.clone(), match props.and_then(|p| p.get(k)).or(extra) {
                    Some(p) => with_defaults(p, v, defs)?,
                    None => v.clone(),
                }));
            }
            if let Some(Json::Arr(required)) = s.get("required") {
                for k in required.iter().filter_map(Json::as_str) {
                    if value.get(k).is_some() {
                        continue;
                    }
                    if let Some(p) = props.and_then(|p| p.get(k)) {
                        if let Some(d) = resolve(p, defs)?.get("default") {
                            out.push((k.to_string(), d.clone()));
                        }
                    }
                }
            }
            Ok(Json::Obj(out))
        }
        _ => Ok(value.clone()),
    }
}

/// A value as JavaScript's `JSON.stringify` writes it.
fn show(v: &Json) -> String {
    v.text()
}

/// A value as JavaScript's `String(…)` writes it (a string unquoted).
fn plain_text(v: &Json) -> String {
    match v {
        Json::Str(s) => s.clone(),
        Json::Arr(items) => items.iter().map(plain_text).collect::<Vec<_>>().join(","),
        Json::Obj(_) => "[object Object]".into(),
        other => other.text(),
    }
}

/// One issue as an English sentence: "distance: must be greater than 0 (got -1)" — the TypeScript `describeIssue`.
pub fn describe_issue(issue: &Issue) -> String {
    let at = if issue.path.is_empty() { String::new() } else { format!("{}: ", issue.path) };
    let got = issue.actual.as_ref().map(|a| format!(" (got {})", show(a))).unwrap_or_default();
    let e = issue.expected.as_ref().map(plain_text).unwrap_or_default();
    match issue.code {
        IssueCode::Required => format!("{at}is required"),
        IssueCode::Type => format!("{at}must be {e}{got}"),
        IssueCode::Integer => format!("{at}must be a whole number{got}"),
        IssueCode::Enum => {
            let values = match &issue.expected {
                Some(Json::Arr(items)) => items.iter().map(plain_text).collect::<Vec<_>>().join(", "),
                _ => e,
            };
            format!("{at}must be one of {values}{got}")
        }
        IssueCode::Const => format!("{at}must be {}{got}", issue.expected.as_ref().map(show).unwrap_or_default()),
        IssueCode::Minimum => format!("{at}must be at least {e}{got}"),
        IssueCode::Maximum => format!("{at}must be at most {e}{got}"),
        IssueCode::ExclusiveMinimum => format!("{at}must be greater than {e}{got}"),
        IssueCode::ExclusiveMaximum => format!("{at}must be less than {e}{got}"),
        IssueCode::MinItems => format!("{at}needs at least {e} items{got}"),
        IssueCode::MaxItems => format!("{at}takes at most {e} items{got}"),
        IssueCode::Additional => format!("{at}is not one of its fields"),
        IssueCode::Pattern => format!("{at}has the wrong form (it must match {e}){got}"),
        IssueCode::AnyOf => format!("{at}does not match an allowed shape{got}"),
    }
}
