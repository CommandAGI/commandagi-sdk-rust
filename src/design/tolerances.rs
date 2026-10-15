//! TOLERANCES — what a toleranced size allows, and the fit of a hole and a shaft. The same rules and messages as the
//! TypeScript SDK's `tolerances.ts`, the Python SDK's `tolerances.py` and the 3D engine's
//! `packages/domain/3d-core/tolerances.ts`:
//!
//! ```
//! use commandagi::design::{tolerances::*, Json};
//!
//! let hole = tolerance_limits(25.0, &Json::parse(r#"{"class": "H7"}"#).unwrap()).unwrap();
//! let shaft = tolerance_limits(25.0, &Json::parse(r#"{"class": "g6"}"#).unwrap()).unwrap();
//! assert_eq!(fit_of(&hole, &shaft).kind, "clearance");
//! ```
//!
//! A class is read from the shipped tables (`cloud://global/standards/iso-286.csv`, `iso-492-normal.csv`;
//! `standards_data.json` `tolerances`): an ISO 286 class (`H7`, `g6`) or a bearing ring's ISO 492 Normal class
//! (`ISO 492 bore`, `ISO 492 outside`, `ISO 492 width`). A class or a size the tables do not have is refused.

use super::json::Json;
use super::standards::standards;

/// What a tolerance allows at a nominal size, in millimetres.
#[derive(Clone, Debug, PartialEq)]
pub struct Limits {
    pub nominal: f64,
    pub upper: f64,
    pub lower: f64,
    pub min: f64,
    pub max: f64,
    pub class: Option<String>,
    pub source: Option<String>,
    pub basic: bool,
}

/// The fit of a hole and a shaft.
#[derive(Clone, Debug, PartialEq)]
pub struct Fit {
    /// `clearance`, `transition` or `interference`.
    pub kind: &'static str,
    pub min_clearance: f64,
    pub max_clearance: f64,
}

fn round6(n: f64) -> f64 {
    (n * 1e6).round() / 1e6 + 0.0
}

fn rows(table: &str) -> &'static [Json] {
    match standards().get("tolerances").and_then(|t| t.get(table)) {
        Some(Json::Arr(a)) => a,
        _ => &[],
    }
}

fn text(row: &Json, key: &str) -> String {
    match row.get(key) {
        Some(Json::Str(s)) => s.clone(),
        Some(Json::Num(n)) => format!("{n}"),
        _ => String::new(),
    }
}

fn num(row: &Json, key: &str) -> f64 {
    row.get(key).and_then(Json::as_f64).unwrap_or(f64::NAN)
}

/// The classes the shipped tables have.
pub fn tolerance_classes() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for r in rows("iso-286") {
        let c = text(r, "class");
        if !out.contains(&c) {
            out.push(c);
        }
    }
    for r in rows("iso-492-normal") {
        let c = format!("ISO 492 {}", text(r, "ring"));
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

/// The bearing ring an "ISO 492 bore|outside|width" name says, lower-case.
fn bearing_ring(name: &str) -> Option<String> {
    let parts: Vec<&str> = name.split_whitespace().collect();
    match parts.as_slice() {
        [iso, n, ring] if iso.eq_ignore_ascii_case("ISO") && *n == "492" => {
            let r = ring.to_lowercase();
            ["bore", "outside", "width"].contains(&r.as_str()).then_some(r)
        }
        _ => None,
    }
}

/// What is wrong with a tolerance as written, or None.
pub fn tolerance_problem(tol: &Json) -> Option<String> {
    let Json::Obj(entries) = tol else {
        return Some("a tolerance is { upper, lower }, { class } or { basic: true }".into());
    };
    for (k, _) in entries {
        if !["upper", "lower", "class", "basic"].contains(&k.as_str()) {
            return Some(format!("a tolerance has no field {k} (it has upper, lower, class, basic)"));
        }
    }
    let (up, lo, cls, basic) = (tol.get("upper"), tol.get("lower"), tol.get("class"), tol.get("basic"));
    if up.is_some() && !matches!(up, Some(Json::Num(_))) {
        return Some("upper is a number of millimetres".into());
    }
    if lo.is_some() && !matches!(lo, Some(Json::Num(_))) {
        return Some("lower is a number of millimetres".into());
    }
    if cls.is_some() && !matches!(cls, Some(Json::Str(s)) if !s.trim().is_empty()) {
        return Some("class is a name (H7, g6, ISO 492 bore)".into());
    }
    if basic.is_some() && !matches!(basic, Some(Json::Bool(_))) {
        return Some("basic is true or false".into());
    }
    let devs = up.is_some() || lo.is_some();
    let basic = matches!(basic, Some(Json::Bool(true)));
    if basic && (devs || cls.is_some()) {
        return Some("a basic dimension has no tolerance of its own: clear basic, or clear the tolerance".into());
    }
    if devs {
        if let Some(Json::Str(c)) = cls {
            return Some(format!("give the class ({c}) or the deviations, not both"));
        }
        let (Some(Json::Num(u)), Some(Json::Num(l))) = (up, lo) else {
            return Some("give both deviations, upper and lower (0 for none)".into());
        };
        if !(u >= l) {
            return Some(format!("the upper deviation ({u}) is below the lower ({l})"));
        }
    }
    if !devs && cls.is_none() && !basic {
        return Some("an empty tolerance: give upper and lower, or a class, or basic".into());
    }
    None
}

fn fmt(n: f64) -> String {
    format!("{}", (n * 1e4).round() / 1e4 + 0.0)
}

/// The sizes a tolerance allows at a nominal size, or why it cannot say.
pub fn tolerance_limits(nominal: f64, tol: &Json) -> Result<Limits, String> {
    if let Some(p) = tolerance_problem(tol) {
        return Err(p);
    }
    if matches!(tol.get("basic"), Some(Json::Bool(true))) {
        return Ok(Limits { nominal, upper: 0.0, lower: 0.0, min: nominal, max: nominal, class: None, source: None, basic: true });
    }
    if let Some(Json::Str(cls)) = tol.get("class") {
        let name = cls.trim();
        let candidates: Vec<&Json> = match bearing_ring(name) {
            Some(ring) => rows("iso-492-normal").iter().filter(|r| text(r, "ring") == ring).collect(),
            None => rows("iso-286").iter().filter(|r| text(r, "class") == name).collect(),
        };
        if candidates.is_empty() {
            return Err(format!("the tables have no tolerance class \"{cls}\" (they have {})", tolerance_classes().join(", ")));
        }
        let Some(row) = candidates.iter().find(|r| nominal > num(r, "overMm") && nominal <= num(r, "uptoMm")) else {
            let lo = candidates.iter().map(|r| num(r, "overMm")).fold(f64::INFINITY, f64::min);
            let hi = candidates.iter().map(|r| num(r, "uptoMm")).fold(f64::NEG_INFINITY, f64::max);
            return Err(format!("{name} is tabled over {} up to {} mm, not at {} mm", fmt(lo), fmt(hi), fmt(nominal)));
        };
        let (upper, lower) = (num(row, "upperUm") / 1000.0, num(row, "lowerUm") / 1000.0);
        let source = match row.get("source") {
            Some(Json::Str(s)) => Some(s.clone()),
            _ => None,
        };
        return Ok(Limits { nominal, upper, lower, min: round6(nominal + lower), max: round6(nominal + upper), class: Some(name.to_string()), source, basic: false });
    }
    let u = tol.get("upper").and_then(Json::as_f64).unwrap_or_default();
    let l = tol.get("lower").and_then(Json::as_f64).unwrap_or_default();
    Ok(Limits { nominal, upper: u, lower: l, min: round6(nominal + l), max: round6(nominal + u), class: None, source: None, basic: false })
}

/// The fit of a hole and a shaft: clearance when the smallest hole takes the largest shaft (a minimum clearance of 0
/// counts), interference when the largest hole is smaller than the smallest shaft, else transition.
pub fn fit_of(hole: &Limits, shaft: &Limits) -> Fit {
    let min_c = hole.min - shaft.max;
    let max_c = hole.max - shaft.min;
    let kind = if min_c >= -1e-9 {
        "clearance"
    } else if max_c <= 1e-9 {
        "interference"
    } else {
        "transition"
    };
    Fit { kind, min_clearance: round6(min_c), max_clearance: round6(max_c) }
}
