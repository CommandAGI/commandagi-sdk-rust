//! THE SCHEMA OF AN ELEMENT'S PROPS, AND ITS CHECK — the TypeScript SDK's `schema.ts`, in Rust. A document's elements
//! (`extrude().distance(10)`) take fields whose JSON Schema is generated from the types that declare them
//! (`threed_schema.json`, scripts/workbench/schemas.mjs in the CommandAGI repository). The reader checks each element
//! against it and refuses a wrong value with the sentence the TypeScript reader, the op panel and the agent's tool give:
//! `distance: must be greater than 0 (got -1)`.
//!
//! The subset: type, enum, const, properties, required, additionalProperties, items, prefixItems, minItems, maxItems,
//! minimum, maximum, exclusiveMinimum, exclusiveMaximum, anyOf, oneOf, `$ref` to `#/$defs/<name>`. `pattern` is not
//! read here: no generated schema of this crate has one (a test holds that), and the crate has no regex engine.

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
        IssueCode::AnyOf => format!("{at}does not match an allowed shape{got}"),
    }
}
